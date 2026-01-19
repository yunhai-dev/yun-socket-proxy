use crate::error::Result;
use crate::protocol::{Address, Reply, SOCKS_VERSION};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::trace;

const RSV: u8 = 0x00;
const ATYP_IPV4: u8 = 0x01;

/// 发送成功响应
///
/// 格式: [VER(1) | REP(1) | RSV(1) | ATYP(1) | BND.ADDR(变长) | BND.PORT(2)]
pub async fn send_success(stream: &mut TcpStream, _address: &Address) -> Result<()> {
    send_reply(stream, Reply::Succeeded).await
}

/// 发送失败响应
pub async fn send_failure(stream: &mut TcpStream, reply: Reply) -> Result<()> {
    send_reply(stream, reply).await
}

/// 发送响应
async fn send_reply(stream: &mut TcpStream, reply: Reply) -> Result<()> {
    trace!("Sending reply: {:?}", reply);

    // 写入版本号
    stream.write_u8(SOCKS_VERSION).await?;

    // 写入响应码
    stream.write_u8(reply as u8).await?;

    // 写入保留字节
    stream.write_u8(RSV).await?;

    // 写入绑定地址（使用 0.0.0.0:0 作为占位符）
    stream.write_u8(ATYP_IPV4).await?;
    stream.write_all(&[0, 0, 0, 0]).await?;
    stream.write_u16(0).await?;

    stream.flush().await?;

    Ok(())
}

/// 从 SocketAddr 获取绑定地址信息
pub fn get_bind_address(addr: &SocketAddr) -> (Vec<u8>, u16) {
    match addr {
        SocketAddr::V4(addr) => {
            let octets = addr.ip().octets();
            (octets.to_vec(), addr.port())
        }
        SocketAddr::V6(addr) => {
            let octets = addr.ip().octets();
            (octets.to_vec(), addr.port())
        }
    }
}
