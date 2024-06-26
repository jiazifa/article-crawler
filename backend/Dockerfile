FROM rust:1.79-buster AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

WORKDIR /app

COPY crates/libs /app/crates/libs
COPY crates/migration /app/crates/migration
COPY crates/services /app/crates/services

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

# 设置国内源为清华大学的源
RUN echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ buster main" > /etc/apt/sources.list \
    && echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ buster-updates main" >> /etc/apt/sources.list \
    && echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ buster-backports main" >> /etc/apt/sources.list \
    && echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian-security buster/updates main" >> /etc/apt/sources.list

RUN cargo build --release -p crawler
RUN cargo build --release -p web-server

# 最终使用 镜像 runner
FROM debian:buster-slim AS runner

# 安装 libssl1.1
RUN apt-get update --allow-releaseinfo-change && apt-get install -y ca-certificates && update-ca-certificates \
    && apt-get install -y libssl1.1
# 创建 /bin 目录
RUN mkdir -p /app/bin

WORKDIR /app

# copy build file from builder image
COPY --from=builder /app/target/release/crawler /app/bin/crawler
COPY --from=builder /app/target/release/web-server /app/bin/web-server

RUN chmod +x /app/bin/*

# Add bin to PATH
ENV PATH="/app/bin:${PATH}"

# copy config file
COPY fixture /app/fixture

# 配置环境变量
ENV server.name="article-crawler"
ENV web.address="0.0.0.0:8001"
ENV web.compression=true
ENV web.cors=true

# 配置 日志
ENV log.dir="/app/logs"
ENV log.level="DEBUG"
ENV log.enable_oper_log=true

# 配置 数据库
ENV database.uri="sqlite:///app/data/article.sqlite3?mode=rwc"

# 配置 其他服务 端点
ENV services.js_server_host="http://localhost:3000"
ENV services.web_api_host="http://localhost:8888"

VOLUME [ "/app/data", "/app/logs" ]

