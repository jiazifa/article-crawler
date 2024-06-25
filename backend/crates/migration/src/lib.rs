pub use sea_orm_migration::prelude::*;
mod m20220101_000001_create_table;
mod m20231211_090712_add_link_meta_columns;
mod m20231219_094918_add_rss_subscrip_count_offset;
mod m20240110_065914_add_rss_link_summary;
mod m20240130_083100_add_rss_link_pure_txt;
mod m20240218_030941_add_rss_index;
mod m20240221_025803_add_rss_update_record;
mod m20240227_070206_add_rss_update_config;
mod m20240320_030818_add_rss_tag_meta;
mod m20240402_033409_add_user_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20231211_090712_add_link_meta_columns::Migration),
            Box::new(m20231219_094918_add_rss_subscrip_count_offset::Migration),
            Box::new(m20240110_065914_add_rss_link_summary::Migration),
            Box::new(m20240130_083100_add_rss_link_pure_txt::Migration),
            Box::new(m20240218_030941_add_rss_index::Migration),
            Box::new(m20240221_025803_add_rss_update_record::Migration),
            Box::new(m20240227_070206_add_rss_update_config::Migration),
            Box::new(m20240320_030818_add_rss_tag_meta::Migration),
            Box::new(m20240402_033409_add_user_table::Migration),
        ]
    }
}
