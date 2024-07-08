#[macro_use]
extern crate derive_builder;
pub mod auth;
pub mod common_schema;
pub mod error;
pub mod rss;

use std::time::Duration;

use error::ErrorInService;
use sea_orm::entity::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, ConnectionTrait, Database};

pub use sea_orm::DbErr;
pub type DBConnection = DatabaseConnection;

pub async fn get_db_conn(uri: String) -> DBConnection {
    let mut opt = ConnectOptions::new(uri.to_owned());
    opt.max_connections(1000)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);
    let db = Database::connect(opt)
        .await
        .expect(format!("Failed to connect to database: {}", uri).as_str());
    tracing::info!("Database connected");
    db
}

#[cfg(test)]
mod test_runner {
    use migration::{Migrator, MigratorTrait};
    use sqlx::migrate::Migrate;

    use super::DBConnection;

    pub(crate) async fn setup_database() -> DBConnection {
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let db = crate::get_db_conn(base_url).await;
        // 假设 `Migrator` 是你的迁移模块，它实现了 `MigratorTrait`
        if let Err(e) = Migrator::up(&db, None).await {
            println!("error: {:?}", e);
        }
        db
    }
}
