pub use sea_orm_migration::prelude::*;
mod m20220101_000001_create_table;
mod m20240110_065914_add_feed_link_summary;
mod m20240221_025803_add_feed_update_record;
mod m20240227_070206_add_feed_update_config;
mod m20240402_033409_add_account_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240110_065914_add_feed_link_summary::Migration),
            Box::new(m20240221_025803_add_feed_update_record::Migration),
            Box::new(m20240227_070206_add_feed_update_config::Migration),
            Box::new(m20240402_033409_add_account_table::Migration),
        ]
    }
}
