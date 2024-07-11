use lib_core::error::ErrorInService;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum APIError {
    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("{0}")]
    ErrorParams(String),

    #[error("{0}")]
    Toast(String),
}

impl From<ErrorInService> for APIError {
    fn from(e: ErrorInService) -> Self {
        match e {
            ErrorInService::Custom(s) => APIError::Toast(s),
            ErrorInService::DBError(s) => APIError::Internal(format!("数据库错误:{}", s)),
            ErrorInService::ErrorInRss(rss_e) => match rss_e {
                lib_core::error::RssError::RssSubscriptionNotFound => {
                    APIError::Toast("订阅不存在".to_string())
                }
                lib_core::error::RssError::RssSubscriptionAlreadyExists => {
                    APIError::Toast("订阅已存在".to_string())
                }
            },
        }
    }
}
