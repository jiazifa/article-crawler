use clap::Parser;
use lib_utils::Setting;
use migration::{Migrator, MigratorTrait};
use std::error::Error;
use std::{net::SocketAddr, str::FromStr};
use tokio::runtime::Builder;
use tracing::level_filters::LevelFilter;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    pub cfg_file: Option<String>,
}

pub fn app() -> Cli {
    Cli::parse()
}

pub struct Server;

impl Server {
    pub fn bind_address(setting: &Setting) -> Result<SocketAddr, Box<dyn Error>> {
        tracing::info!("将绑定地址: {}", setting.web.address);
        let addr = SocketAddr::from_str(setting.web.address.as_str())?;
        Ok(addr)
    }

    pub fn run(&self, cli: Cli) -> Result<(), Box<dyn Error>> {
        let runtime = Builder::new_multi_thread().enable_all().build()?;
        runtime.block_on(async {
            let setting = match Setting::from_config(cli.cfg_file) {
                Ok(setting) => setting,
                Err(e) => {
                    println!("配置文件解析失败:{}， 将使用默认配置运行", e);
                    Setting::default()
                }
            };
            Setting::set_global(setting.clone());

            let setting = Setting::global();

            let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::TRACE.into())
                    .parse_lossy("")
            });

            // 输出到控制台中
            let formatting_layer = fmt::layer()
                .with_level(true)
                .pretty()
                .with_writer(std::io::stderr);

            // 输出到文件中
            let log_file_path = format!("{}.log", chrono::Local::now().format("%Y-%m-%d"));
            let file_appender = rolling::never(
                setting.log.dir.clone().unwrap_or("logs".to_string()),
                log_file_path,
            );
            let (non_blocking_appender, _guard) = non_blocking(file_appender);
            let file_layer = fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking_appender)
                .with_span_events(fmt::format::FmtSpan::CLOSE);

            // 注册
            Registry::default()
                .with(env_filter)
                // ErrorLayer 可以让 color-eyre 获取到 span 的信息
                .with(formatting_layer)
                .with(file_layer)
                .init();

            let router = web_server::build_router(&setting).await;

            let addr = match Self::bind_address(&setting) {
                Ok(addr) => addr,
                Err(e) => {
                    tracing::error!("绑定地址失败:{}", e);
                    std::process::exit(1);
                }
            };
            tracing::info!(
                "将开始在: {:?} 创建数据库连接",
                &setting.database.uri.clone()
            );
            let conn = lib_core::get_db_conn(setting.database.uri.clone()).await;
            if let Err(e) = Migrator::up(&conn, None).await {
                tracing::error!("数据库迁移失败:{}", e);
            }

            tracing::info!("将开始在: {:?} 创建服务", &addr);
            let tcp_listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(tcp_listener, router.into_make_service())
                .await
                .unwrap();
        });
        Ok(())
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let server = Server;
    let mut app = Cli::parse();
    // if debug, set RUST_LOG=debug
    app.cfg_file = Some("fixture/config.toml".to_string());
    server.run(app)?;
    Ok(())
}
