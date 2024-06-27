use derive_builder::UninitializedFieldError;
use sea_orm::DbErr;
use sqlx::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RssError {
    #[error("rss subscription not found")]
    RssSubscriptionNotFound,
    #[error("rss subscription already exists")]
    RssSubscriptionAlreadyExists,
}

#[derive(Error, Debug)]
pub enum ErrorInService {
    #[error("rss `{0}`")]
    ErrorInRss(#[from] RssError),
    #[error("`{0}`")]
    Custom(String),
    #[error("`{0}`")]
    DBError(DbErr),
}

impl From<DbErr> for ErrorInService {
    fn from(e: DbErr) -> Self {
        Self::DBError(e)
    }
}

impl From<UninitializedFieldError> for ErrorInService {
    fn from(value: UninitializedFieldError) -> Self {
        Self::Custom(format!("UninitializedFieldError: {}", value))
    }
}
