use crate::error::Result;
use tokio::io;
use tokio::net::TcpStream;
use tracing::{debug, trace};

/// 双向数据转发
///
/// 使用 tokio 的零拷贝实现在客户端和目标服务器之间转发数据
pub async fn bidirectional_copy(
    mut client: TcpStream,
    mut target: TcpStream,
) -> Result<(u64, u64)> {
    trace!("Starting bidirectional data relay");

    // 使用 tokio 的零拷贝双向转发
    let (client_to_target, target_to_client) =
        io::copy_bidirectional(&mut client, &mut target).await?;

    debug!(
        "Connection closed - Client->Target: {} bytes, Target->Client: {} bytes",
        client_to_target, target_to_client
    );

    Ok((client_to_target, target_to_client))
}
