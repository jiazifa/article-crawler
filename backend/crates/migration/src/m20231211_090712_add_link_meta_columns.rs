use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // Fix Sqlite doesn't support multiple alter options
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    .add_column(ColumnDef::new(Alias::new("images")).text().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    .add_column(ColumnDef::new(Alias::new("authors_json")).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    .drop_column(Alias::new("images"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    .drop_column(Alias::new("authors_json"))
                    .to_owned(),
            )
            .await
    }
}
