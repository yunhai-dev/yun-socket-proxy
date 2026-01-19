use crate::error::{ProxyError, Result};
use crate::protocol::{Address, Command, Request, SOCKS_VERSION};
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::{AsyncReadExt};
use tokio::net::TcpStream;
use tracing::trace;

const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_IPV6: u8 = 0x04;

/// 解析 SOCKS5 请求
///
/// 格式: [VER(1) | CMD(1) | RSV(1) | ATYP(1) | DST.ADDR(变长) | DST.PORT(2)]
pub async fn parse_request(stream: &mut TcpStream) -> Result<Request> {
    // 读取版本号
    let version = stream.read_u8().await?;
    if version != SOCKS_VERSION {
        return Err(ProxyError::InvalidVersion(version));
    }

    // 读取命令
    let cmd = stream.read_u8().await?;
    let command = Command::from_u8(cmd)
        .ok_or(ProxyError::UnsupportedCommand(cmd))?;

    // 读取保留字节
    let _rsv = stream.read_u8().await?;

    // 读取地址类型
    let atyp = stream.read_u8().await?;

    // 根据地址类型解析目标地址
    let address = match atyp {
        ATYP_IPV4 => {
            let mut octets = [0u8; 4];
            stream.read_exact(&mut octets).await?;
            let ip = Ipv4Addr::from(octets);
            let port = stream.read_u16().await?;
            Address::Ipv4(ip, port)
        }
        ATYP_IPV6 => {
            let mut octets = [0u8; 16];
            stream.read_exact(&mut octets).await?;
            let ip = Ipv6Addr::from(octets);
            let port = stream.read_u16().await?;
            Address::Ipv6(ip, port)
        }
        ATYP_DOMAIN => {
            let domain_len = stream.read_u8().await?;
            let mut domain_bytes = vec![0u8; domain_len as usize];
            stream.read_exact(&mut domain_bytes).await?;
            let domain = String::from_utf8(domain_bytes)
                .map_err(|_| ProxyError::InvalidAddress)?;
            let port = stream.read_u16().await?;
            Address::Domain(domain, port)
        }
        _ => return Err(ProxyError::UnsupportedAddressType(atyp)),
    };

    trace!("Parsed request - Command: {:?}, Address: {}", command, address.to_string());

    Ok(Request { command, address })
}
