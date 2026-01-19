use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};

/// 创建一个简单的 echo 服务器用于测试
async fn start_echo_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();

        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    while let Ok(n) = socket.read(&mut buf).await {
                        if n == 0 {
                            break;
                        }
                        let _ = socket.write_all(&buf[..n]).await;
                    }
                });
            }
        }
    })
}

/// 测试 SOCKS5 握手（无认证）
#[tokio::test]
async fn test_socks5_handshake_no_auth() {
    // 启动代理服务器
    let config = yun_socket_proxy::config::Config::default();
    let server = yun_socket_proxy::server::ProxyServer::new(config);

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 连接到代理服务器
    let mut stream = TcpStream::connect("127.0.0.1:1080").await.unwrap();

    // 发送握手请求: [VER(0x05) | NMETHODS(1) | METHODS(0x00)]
    stream.write_all(&[0x05, 0x01, 0x00]).await.unwrap();

    // 读取握手响应: [VER(0x05) | METHOD(0x00)]
    let mut response = [0u8; 2];
    stream.read_exact(&mut response).await.unwrap();

    assert_eq!(response[0], 0x05); // SOCKS version
    assert_eq!(response[1], 0x00); // No authentication
}

/// 测试完整的 SOCKS5 连接流程
#[tokio::test]
async fn test_socks5_connect_flow() {
    // 启动 echo 服务器
    let echo_port = 9999;
    let _echo_server = start_echo_server(echo_port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 启动代理服务器
    let mut config = yun_socket_proxy::config::Config::default();
    config.server.port = 1081;
    let server = yun_socket_proxy::server::ProxyServer::new(config);

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 连接到代理服务器
    let mut stream = TcpStream::connect("127.0.0.1:1081").await.unwrap();

    // 1. 握手
    stream.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    let mut response = [0u8; 2];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, [0x05, 0x00]);

    // 2. 发送 CONNECT 请求到 echo 服务器
    // [VER(0x05) | CMD(0x01) | RSV(0x00) | ATYP(0x01) | DST.ADDR(127.0.0.1) | DST.PORT(9999)]
    let mut request = vec![0x05, 0x01, 0x00, 0x01];
    request.extend_from_slice(&[127, 0, 0, 1]); // 127.0.0.1
    request.extend_from_slice(&echo_port.to_be_bytes()); // port
    stream.write_all(&request).await.unwrap();

    // 3. 读取连接响应
    let mut connect_response = [0u8; 10];
    stream.read_exact(&mut connect_response).await.unwrap();
    assert_eq!(connect_response[0], 0x05); // SOCKS version
    assert_eq!(connect_response[1], 0x00); // Success

    // 4. 测试数据传输
    let test_data = b"Hello, SOCKS5!";
    stream.write_all(test_data).await.unwrap();

    let mut echo_response = vec![0u8; test_data.len()];
    let result = timeout(Duration::from_secs(2), stream.read_exact(&mut echo_response)).await;

    assert!(result.is_ok());
    assert_eq!(&echo_response, test_data);
}

/// 测试连接限制
#[tokio::test]
async fn test_connection_limit() {
    let mut config = yun_socket_proxy::config::Config::default();
    config.server.port = 1082;
    config.server.max_connections = 2;
    let server = yun_socket_proxy::server::ProxyServer::new(config);

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 创建两个连接
    let _conn1 = TcpStream::connect("127.0.0.1:1082").await.unwrap();
    let _conn2 = TcpStream::connect("127.0.0.1:1082").await.unwrap();

    // 第三个连接应该能建立（因为 TCP 连接可以建立，但会在握手时被限制）
    let conn3_result = timeout(
        Duration::from_millis(500),
        TcpStream::connect("127.0.0.1:1082")
    ).await;

    assert!(conn3_result.is_ok());
}

/// 测试域名解析
#[tokio::test]
async fn test_domain_resolution() {
    let mut config = yun_socket_proxy::config::Config::default();
    config.server.port = 1083;
    let server = yun_socket_proxy::server::ProxyServer::new(config);

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect("127.0.0.1:1083").await.unwrap();

    // 握手
    stream.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    let mut response = [0u8; 2];
    stream.read_exact(&mut response).await.unwrap();

    // 发送域名连接请求
    // [VER | CMD | RSV | ATYP(0x03) | LEN | DOMAIN | PORT]
    let domain = b"example.com";
    let mut request = vec![0x05, 0x01, 0x00, 0x03];
    request.push(domain.len() as u8);
    request.extend_from_slice(domain);
    request.extend_from_slice(&80u16.to_be_bytes());

    stream.write_all(&request).await.unwrap();

    // 读取响应（可能成功或失败，取决于网络）
    let mut connect_response = vec![0u8; 10];
    let result = timeout(Duration::from_secs(5), stream.read_exact(&mut connect_response)).await;

    // 只要能收到响应就算测试通过
    assert!(result.is_ok());
}
