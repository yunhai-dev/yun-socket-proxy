# 多阶段构建 Dockerfile
# 阶段 1: 构建阶段
FROM rust:1.83-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev

# 创建工作目录
WORKDIR /app

# 复制所有源代码
COPY . .

# 构建 release 版本
RUN cargo build --release

# 阶段 2: 运行阶段
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache ca-certificates

# 创建非 root 用户
RUN addgroup -g 1000 proxy && \
    adduser -D -u 1000 -G proxy proxy

# 创建工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/yun-socket-proxy /usr/local/bin/yun-socket-proxy

# 复制配置文件示例
COPY config.example.toml /app/config.example.toml

# 切换到非 root 用户
USER proxy

# 暴露默认端口
EXPOSE 1080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD nc -z localhost 1080 || exit 1

# 设置入口点
ENTRYPOINT ["/usr/local/bin/yun-socket-proxy"]

# 默认参数
CMD ["--log-level", "info"]

