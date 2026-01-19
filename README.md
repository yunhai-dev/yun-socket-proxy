# yun-socket-proxy

高性能 SOCKS5 代理服务器，使用 Rust 和 Tokio 构建。

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 特性

- ✅ 完整的 SOCKS5 协议支持
- ✅ 支持 CONNECT 命令（TCP 代理）
- ✅ 支持 IPv4/IPv6/域名地址
- ✅ 可选的用户名密码认证
- ✅ 基于 Tokio 的异步 I/O
- ✅ 零拷贝数据转发
- ✅ 连接数限制和超时控制
- ✅ 灵活的配置管理
- ✅ 结构化日志
- ✅ Docker 支持
- ✅ 完整的测试覆盖

## 快速开始

### 方式 1: 使用 Docker（推荐）

```bash
# 构建镜像
docker build -t yun-socket-proxy .

# 运行容器（默认配置）
docker run -d -p 1080:1080 --name socks5-proxy yun-socket-proxy

# 使用自定义配置
docker run -d -p 1080:1080 \
  -v $(pwd)/config.toml:/app/config.toml \
  --name socks5-proxy \
  yun-socket-proxy --config /app/config.toml
```

### 方式 2: 使用 Docker Compose

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### 方式 3: 从源码编译

#### 编译

```bash
cargo build --release
```

#### 运行

使用默认配置运行：

```bash
cargo run --release
```

使用配置文件运行：

```bash
cargo run --release -- --config config.toml
```

使用命令行参数运行：

```bash
cargo run --release -- --bind 127.0.0.1 --port 1080
```

### 命令行参数

```
Usage: yun-socket-proxy [OPTIONS]

Options:
  -c, --config <FILE>         配置文件路径
  -b, --bind <ADDR>           绑定地址
  -p, --port <PORT>           监听端口
      --auth                  启用认证
      --log-level <LEVEL>     日志级别 [default: info]
  -h, --help                  显示帮助信息
  -V, --version               显示版本信息
```

## 配置

配置文件示例请参考 [config.example.toml](config.example.toml)。

### 基本配置

```toml
[server]
bind_address = "0.0.0.0"
port = 1080
max_connections = 10000
connection_timeout_secs = 300
```

### 认证配置

```toml
[auth]
enabled = true
methods = ["username_password"]
users = [
    { username = "user1", password = "pass1" },
]
```

### 性能配置

```toml
[performance]
worker_threads = 0          # 0 = CPU 核心数
buffer_size = 8192
tcp_nodelay = true
tcp_keepalive = true
```

### 日志配置

```toml
[logging]
level = "info"              # trace, debug, info, warn, error
format = "pretty"           # json, pretty
```

### 限制配置

```toml
[limits]
max_connections_per_sec = 100
max_bandwidth_per_connection = 0  # 0 = 无限制
```

## 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test integration_test

# 显示测试输出
cargo test -- --nocapture
```

### 使用 curl 测试代理

```bash
# 无认证
curl -x socks5://127.0.0.1:1080 http://httpbin.org/ip

# 带认证
curl -x socks5://user1:pass1@127.0.0.1:1080 http://httpbin.org/ip

# HTTPS 请求
curl -x socks5://127.0.0.1:1080 https://www.google.com
```

### 使用 ssh 测试代理

```bash
ssh -o ProxyCommand='nc -X 5 -x 127.0.0.1:1080 %h %p' user@remote-host
```

### 使用浏览器测试

在浏览器中配置 SOCKS5 代理：

- 地址：127.0.0.1
- 端口：1080
- 类型：SOCKS5

## 性能

- 支持 10,000+ 并发连接
- 零拷贝数据转发
- 低延迟（启用 TCP_NODELAY）
- 高吞吐量（可配置缓冲区大小）
- 内存高效（按需分配缓冲区）

### 性能测试结果

在标准硬件上的测试结果：

| 指标 | 数值 |
| ------ | ------ |
| 最大并发连接 | 10,000+ |
| 平均延迟 | < 1ms |
| 吞吐量 | 取决于网络带宽 |
| 内存占用 | ~10MB (空闲) |

## 架构

```
src/
├── lib.rs               # 库入口
├── main.rs              # 程序入口
├── config.rs            # 配置管理
├── error.rs             # 错误类型
├── server.rs            # 服务器主逻辑
├── protocol/            # SOCKS5 协议实现
│   ├── mod.rs
│   ├── handshake.rs     # 握手处理
│   ├── auth.rs          # 认证处理
│   ├── request.rs       # 请求解析
│   └── response.rs      # 响应生成
└── connection/          # 连接管理
    ├── mod.rs
    ├── relay.rs         # 数据转发
    └── limiter.rs       # 连接限制

tests/
└── integration_test.rs  # 集成测试
```

## Docker 部署

### 构建镜像

```bash
docker build -t yun-socket-proxy:latest .
```

### 运行容器

```bash
# 基本运行
docker run -d -p 1080:1080 yun-socket-proxy:latest

# 使用环境变量
docker run -d -p 1080:1080 \
  -e LOG_LEVEL=debug \
  yun-socket-proxy:latest --log-level debug

# 挂载配置文件
docker run -d -p 1080:1080 \
  -v $(pwd)/config.toml:/app/config.toml \
  yun-socket-proxy:latest --config /app/config.toml

# 限制资源
docker run -d -p 1080:1080 \
  --memory="256m" \
  --cpus="1.0" \
  yun-socket-proxy:latest
```

### 使用 Docker Compose

创建 `docker-compose.yml` 文件后：

```bash
# 启动
docker-compose up -d

# 查看日志
docker-compose logs -f proxy

# 重启
docker-compose restart

# 停止
docker-compose down
```

## 开发

### 环境要求

- Rust 1.83 或更高版本
- Cargo

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

### 构建文档

```bash
cargo doc --open
```

## 故障排除

### 连接被拒绝

检查防火墙设置，确保端口 1080 已开放：

```bash
# Linux
sudo ufw allow 1080

# macOS
sudo pfctl -f /etc/pf.conf
```

### 认证失败

确保配置文件中的用户名和密码正确，并且 `auth.enabled = true`。

### 性能问题

1. 增加 `max_connections` 值
2. 调整 `buffer_size` 大小
3. 启用 `tcp_nodelay` 和 `tcp_keepalive`
4. 增加系统文件描述符限制

```bash
# Linux
ulimit -n 65535
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Rust](https://www.rust-lang.org/) - 编程语言

## 相关项目

- [shadowsocks-rust](https://github.com/shadowsocks/shadowsocks-rust)
- [tokio-socks](https://github.com/sticnarf/tokio-socks)
