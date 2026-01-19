pub mod handshake;
pub mod auth;
pub mod request;
pub mod response;

use std::net::{Ipv4Addr, Ipv6Addr};

// SOCKS5 协议常量
pub const SOCKS_VERSION: u8 = 0x05;

// 认证方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    NoAuth = 0x00,
    UsernamePassword = 0x02,
    NoAcceptable = 0xFF,
}

impl AuthMethod {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(AuthMethod::NoAuth),
            0x02 => Some(AuthMethod::UsernamePassword),
            0xFF => Some(AuthMethod::NoAcceptable),
            _ => None,
        }
    }
}

// SOCKS5 命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

impl Command {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Command::Connect),
            0x02 => Some(Command::Bind),
            0x03 => Some(Command::UdpAssociate),
            _ => None,
        }
    }
}

// 地址类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Ipv4(Ipv4Addr, u16),
    Ipv6(Ipv6Addr, u16),
    Domain(String, u16),
}

impl Address {
    pub fn port(&self) -> u16 {
        match self {
            Address::Ipv4(_, port) => *port,
            Address::Ipv6(_, port) => *port,
            Address::Domain(_, port) => *port,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Address::Ipv4(ip, port) => format!("{}:{}", ip, port),
            Address::Ipv6(ip, port) => format!("[{}]:{}", ip, port),
            Address::Domain(domain, port) => format!("{}:{}", domain, port),
        }
    }
}

// 响应状态码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reply {
    Succeeded = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
}

// SOCKS5 请求
#[derive(Debug, Clone)]
pub struct Request {
    pub command: Command,
    pub address: Address,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_method_from_u8() {
        assert_eq!(AuthMethod::from_u8(0x00), Some(AuthMethod::NoAuth));
        assert_eq!(AuthMethod::from_u8(0x02), Some(AuthMethod::UsernamePassword));
        assert_eq!(AuthMethod::from_u8(0xFF), Some(AuthMethod::NoAcceptable));
        assert_eq!(AuthMethod::from_u8(0x01), None);
    }

    #[test]
    fn test_command_from_u8() {
        assert_eq!(Command::from_u8(0x01), Some(Command::Connect));
        assert_eq!(Command::from_u8(0x02), Some(Command::Bind));
        assert_eq!(Command::from_u8(0x03), Some(Command::UdpAssociate));
        assert_eq!(Command::from_u8(0x04), None);
    }

    #[test]
    fn test_address_ipv4() {
        let addr = Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1), 8080);
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_address_ipv6() {
        let addr = Address::Ipv6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080);
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.to_string(), "[::1]:8080");
    }

    #[test]
    fn test_address_domain() {
        let addr = Address::Domain("example.com".to_string(), 443);
        assert_eq!(addr.port(), 443);
        assert_eq!(addr.to_string(), "example.com:443");
    }

    #[test]
    fn test_request_creation() {
        let request = Request {
            command: Command::Connect,
            address: Address::Domain("google.com".to_string(), 80),
        };

        assert_eq!(request.command, Command::Connect);
        assert_eq!(request.address.port(), 80);
    }

    #[test]
    fn test_reply_values() {
        assert_eq!(Reply::Succeeded as u8, 0x00);
        assert_eq!(Reply::GeneralFailure as u8, 0x01);
        assert_eq!(Reply::ConnectionRefused as u8, 0x05);
        assert_eq!(Reply::CommandNotSupported as u8, 0x07);
    }
}
