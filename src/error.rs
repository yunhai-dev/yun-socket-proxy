use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid SOCKS version: {0}")]
    InvalidVersion(u8),

    #[error("No acceptable authentication method")]
    NoAcceptableAuth,

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Unsupported command: {0}")]
    UnsupportedCommand(u8),

    #[error("Unsupported address type: {0}")]
    UnsupportedAddressType(u8),

    #[error("Invalid address format")]
    InvalidAddress,

    #[error("Connection refused")]
    ConnectionRefused,

    #[error("Host unreachable")]
    HostUnreachable,

    #[error("Network unreachable")]
    NetworkUnreachable,

    #[error("Connection timeout")]
    Timeout,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, ProxyError>;
