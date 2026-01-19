use crate::config::Config;
use crate::connection::{bidirectional_copy, ConnectionLimiter};
use crate::error::{ProxyError, Result};
use crate::protocol::{self, AuthMethod, Command, Reply};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// SOCKS5 代理服务器
pub struct ProxyServer {
    config: Arc<Config>,
    limiter: ConnectionLimiter,
}

impl ProxyServer {
    pub fn new(config: Config) -> Self {
        let limiter = ConnectionLimiter::new(config.server.max_connections);
        Self {
            config: Arc::new(config),
            limiter,
        }
    }

    /// 启动服务器
    pub async fn run(&self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.server.bind_address, self.config.server.port);
        let listener = TcpListener::bind(&bind_addr).await?;

        info!("SOCKS5 proxy server listening on {}", bind_addr);
        info!("Max connections: {}", self.config.server.max_connections);
        info!("Authentication: {}", if self.config.auth.enabled { "enabled" } else { "disabled" });

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New connection from {}", addr);

                    // 检查连接限制
                    let guard = match self.limiter.acquire().await {
                        Some(guard) => guard,
                        None => {
                            warn!("Connection limit reached, rejecting connection from {}", addr);
                            continue;
                        }
                    };

                    let config = self.config.clone();

                    // 为每个连接创建独立的异步任务
                    tokio::spawn(async move {
                        let _guard = guard; // 保持守卫直到任务结束

                        if let Err(e) = handle_client(stream, config).await {
                            error!("Error handling client {}: {}", addr, e);
                        }

                        debug!("Connection from {} closed", addr);
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

/// 处理客户端连接
async fn handle_client(mut client_stream: TcpStream, config: Arc<Config>) -> Result<()> {
    // 设置 TCP 选项
    if config.performance.tcp_nodelay {
        client_stream.set_nodelay(true)?;
    }

    // 1. 握手阶段 - 协商认证方法
    let auth_method = protocol::handshake::negotiate_auth(
        &mut client_stream,
        config.auth.enabled,
    ).await?;

    // 2. 认证阶段（如果需要）
    if auth_method == AuthMethod::UsernamePassword {
        protocol::auth::authenticate(&mut client_stream, &config.auth).await?;
    }

    // 3. 请求阶段 - 解析目标地址
    let request = protocol::request::parse_request(&mut client_stream).await?;

    // 4. 处理命令
    match request.command {
        Command::Connect => {
            handle_connect(client_stream, request.address, config).await
        }
        Command::Bind => {
            protocol::response::send_failure(&mut client_stream, Reply::CommandNotSupported).await?;
            Err(ProxyError::UnsupportedCommand(Command::Bind as u8))
        }
        Command::UdpAssociate => {
            protocol::response::send_failure(&mut client_stream, Reply::CommandNotSupported).await?;
            Err(ProxyError::UnsupportedCommand(Command::UdpAssociate as u8))
        }
    }
}

/// 处理 CONNECT 命令
async fn handle_connect(
    mut client_stream: TcpStream,
    address: protocol::Address,
    config: Arc<Config>,
) -> Result<()> {
    info!("Connecting to {}", address.to_string());

    // 连接到目标服务器
    let target_addr = address.to_string();
    let connect_timeout = Duration::from_secs(config.server.connection_timeout_secs);

    let target_stream = match timeout(connect_timeout, TcpStream::connect(&target_addr)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            error!("Failed to connect to {}: {}", target_addr, e);
            let reply = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => Reply::ConnectionRefused,
                std::io::ErrorKind::TimedOut => Reply::TtlExpired,
                _ => Reply::HostUnreachable,
            };
            protocol::response::send_failure(&mut client_stream, reply).await?;
            return Err(ProxyError::from(e));
        }
        Err(_) => {
            error!("Connection timeout to {}", target_addr);
            protocol::response::send_failure(&mut client_stream, Reply::TtlExpired).await?;
            return Err(ProxyError::Timeout);
        }
    };

    // 设置目标连接的 TCP 选项
    if config.performance.tcp_nodelay {
        target_stream.set_nodelay(true)?;
    }

    // 发送成功响应
    protocol::response::send_success(&mut client_stream, &address).await?;

    info!("Successfully connected to {}", target_addr);

    // 双向数据转发
    match bidirectional_copy(client_stream, target_stream).await {
        Ok((client_to_target, target_to_client)) => {
            debug!(
                "Data transfer completed - Sent: {} bytes, Received: {} bytes",
                client_to_target, target_to_client
            );
            Ok(())
        }
        Err(e) => {
            error!("Error during data transfer: {}", e);
            Err(e)
        }
    }
}
