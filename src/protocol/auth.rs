use crate::config::AuthConfig;
use crate::error::{ProxyError, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, warn};

const USERNAME_PASSWORD_VERSION: u8 = 0x01;
const AUTH_SUCCESS: u8 = 0x00;
const AUTH_FAILURE: u8 = 0x01;

/// 处理用户名密码认证
///
/// 客户端发送: [VER(1) | ULEN(1) | UNAME(1-255) | PLEN(1) | PASSWD(1-255)]
/// 服务器响应: [VER(1) | STATUS(1)]
pub async fn authenticate(stream: &mut TcpStream, config: &AuthConfig) -> Result<()> {
    // 读取认证版本
    let version = stream.read_u8().await?;
    if version != USERNAME_PASSWORD_VERSION {
        return Err(ProxyError::Protocol(format!("Invalid auth version: {}", version)));
    }

    // 读取用户名
    let username_len = stream.read_u8().await?;
    let mut username_bytes = vec![0u8; username_len as usize];
    stream.read_exact(&mut username_bytes).await?;
    let username = String::from_utf8_lossy(&username_bytes).to_string();

    // 读取密码
    let password_len = stream.read_u8().await?;
    let mut password_bytes = vec![0u8; password_len as usize];
    stream.read_exact(&mut password_bytes).await?;
    let password = String::from_utf8_lossy(&password_bytes).to_string();

    debug!("Authentication attempt for user: {}", username);

    // 验证用户名和密码
    let authenticated = config.users.iter().any(|user| {
        user.username == username && user.password == password
    });

    // 发送认证结果
    stream.write_u8(USERNAME_PASSWORD_VERSION).await?;
    if authenticated {
        stream.write_u8(AUTH_SUCCESS).await?;
        stream.flush().await?;
        debug!("Authentication successful for user: {}", username);
        Ok(())
    } else {
        stream.write_u8(AUTH_FAILURE).await?;
        stream.flush().await?;
        warn!("Authentication failed for user: {}", username);
        Err(ProxyError::AuthFailed)
    }
}
