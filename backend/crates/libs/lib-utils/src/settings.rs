use std::{env, sync::OnceLock};

use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::cell::OnceCell;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Jwt {
    pub secret: String,
    pub exp: i64,
}

impl Default for Jwt {
    fn default() -> Self {
        Self {
            secret: "im_fake_secret".to_string(),
            exp: 60 * 60 * 24 * 90,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Web {
    pub address: String,
    pub compression: Option<bool>,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:3000".to_string(),
            compression: Some(true),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Log {
    pub dir: Option<String>,
    pub level: Option<String>,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            dir: Some("logs".to_string()),
            level: Some("info".to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAI {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
}

impl Default for OpenAI {
    fn default() -> Self {
        Self {
            api_key: match env::var("openai.api_key") {
                Ok(api_key) => Some(api_key),
                Err(_) => None,
            },
            api_base: match env::var("openai.api_base") {
                Ok(api_base) => Some(api_base),
                Err(_) => None,
            },
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Services {
    pub js_server_host: Option<String>,
    pub web_api_host: Option<String>,
}

impl Default for Services {
    fn default() -> Self {
        Self {
            js_server_host: Some(
                env::var("services.js_server_host")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            ),
            web_api_host: Some(
                env::var("services.web_api_host")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Setting {
    pub debug: bool,
    pub database: Database,
    pub jwt: Jwt,
    pub web: Web,
    pub log: Log,
    pub openai: OpenAI,
    pub services: Services,
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            debug: true,
            database: Database {
                uri: "sqlite::memory:?mode=rwc".to_string(),
            },
            jwt: Jwt::default(),
            web: Web::default(),
            log: Log::default(),
            openai: OpenAI::default(),
            services: Services::default(),
        }
    }
}

#[cfg(test)]
impl Setting {
    pub fn test() -> Setting {
        Setting::default()
    }
}

static SHARED_SETTING: OnceLock<Setting> = OnceLock::new();

impl Setting {
    pub fn global() -> Setting {
        assert!(SHARED_SETTING.get().is_some());
        SHARED_SETTING.get().expect("配置文件未加载").clone()
    }

    pub fn from_config(file: Option<String>) -> Result<Setting, ConfigError> {
        let mut cfg = load_configure(file);
        let setting = cfg.try_deserialize::<Setting>()?;
        Ok(setting)
    }

    pub fn set_global(setting: Setting) {
        SHARED_SETTING.set(setting.clone()).expect("配置文件未加载");
        let cloned = setting.clone();
        SHARED_SETTING.get_or_init(|| cloned);
    }
}

fn load_configure(file: Option<String>) -> Config {
    let mut builder = Config::builder();
    // set debug to true
    builder = builder.set_default("debug", true).unwrap();

    builder = builder.set_default("web.address", "0.0.0.0:8888").unwrap();
    builder = builder.set_default("web.compression", true).unwrap();

    builder = builder.set_default("log.level", "info").unwrap();
    builder = builder.set_default("log.dir", "logs").unwrap();

    builder = builder.set_default("jwt.secret", "secret").unwrap();
    builder = builder.set_default("jwt.exp", 3600 * 24 * 30).unwrap();

    builder = builder
        .set_default("database.uri", "sqlite://data.db?mode=rwc")
        .unwrap();
    if let Some(cfg_file) = file.as_ref() {
        if std::path::Path::new(&cfg_file).exists() {
            builder = builder.add_source(config::File::with_name(cfg_file).required(false));
        } else {
            println!("配置文件不存在，将使用默认配置{}运行", cfg_file);
        }
    } else {
        builder = builder.add_source(Environment::default());
    }

    builder.build().expect("配置文件加载失败")
}
