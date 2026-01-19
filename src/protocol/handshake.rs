use crate::error::{ProxyError, Result};
use crate::protocol::{AuthMethod, SOCKS_VERSION};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, trace};

/// 处理 SOCKS5 握手，协商认证方法
///
/// 客户端发送: [VER(1) | NMETHODS(1) | METHODS(1-255)]
/// 服务器响应: [VER(1) | METHOD(1)]
pub async fn negotiate_auth(stream: &mut TcpStream, auth_enabled: bool) -> Result<AuthMethod> {
    // 读取版本号
    let version = stream.read_u8().await?;
    if version != SOCKS_VERSION {
        return Err(ProxyError::InvalidVersion(version));
    }

    // 读取认证方法数量
    let nmethods = stream.read_u8().await?;
    if nmethods == 0 {
        return Err(ProxyError::Protocol("No authentication methods provided".to_string()));
    }

    // 读取所有认证方法
    let mut methods = vec![0u8; nmethods as usize];
    stream.read_exact(&mut methods).await?;

    trace!("Client offered auth methods: {:?}", methods);

    // 选择认证方法
    let selected_method = if auth_enabled {
        // 如果启用认证，优先选择用户名密码认证
        if methods.contains(&(AuthMethod::UsernamePassword as u8)) {
            AuthMethod::UsernamePassword
        } else {
            AuthMethod::NoAcceptable
        }
    } else {
        // 如果未启用认证，选择无认证
        if methods.contains(&(AuthMethod::NoAuth as u8)) {
            AuthMethod::NoAuth
        } else {
            AuthMethod::NoAcceptable
        }
    };

    // 发送选择的认证方法
    stream.write_u8(SOCKS_VERSION).await?;
    stream.write_u8(selected_method as u8).await?;
    stream.flush().await?;

    debug!("Selected auth method: {:?}", selected_method);

    if selected_method == AuthMethod::NoAcceptable {
        return Err(ProxyError::NoAcceptableAuth);
    }

    Ok(selected_method)
}
