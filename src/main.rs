use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use yun_socket_proxy::{Config, ProxyServer};

#[derive(Parser, Debug)]
#[command(name = "yun-socket-proxy")]
#[command(author, version, about = "High-performance SOCKS5 proxy server", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Bind address
    #[arg(short, long, value_name = "ADDR")]
    bind: Option<String>,

    /// Port to listen on
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    /// Enable authentication
    #[arg(long)]
    auth: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 初始化日志
    init_logging(&args.log_level);

    info!("Starting yun-socket-proxy v{}", env!("CARGO_PKG_VERSION"));

    // 加载配置
    let mut config = if let Some(config_path) = args.config {
        match Config::from_file(&config_path) {
            Ok(config) => {
                info!("Loaded configuration from {:?}", config_path);
                config
            }
            Err(e) => {
                error!("Failed to load configuration file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        info!("Using default configuration");
        Config::default()
    };

    // 命令行参数覆盖配置文件
    if let Some(bind) = args.bind {
        config.server.bind_address = bind;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if args.auth {
        config.auth.enabled = true;
    }

    // 创建并运行服务器
    let server = ProxyServer::new(config);

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}

fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(filter)
        .init();
}
