use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("rss_subscrip_count_offset"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("subscription_id"))
                            .integer()
                            .primary_key()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("offset")).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("rss_subscrip_count_offset"))
                    .to_owned(),
            )
            .await
    }
}
