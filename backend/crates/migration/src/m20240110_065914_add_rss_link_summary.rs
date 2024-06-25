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
                    .table(Alias::new("rss_link_summary"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("link_url"))
                            .string_len(32u32)
                            .primary_key()
                            .unique_key()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("version"))
                            .string_len(32u32)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("provider"))
                            .string_len(32u32)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("language"))
                            .string_len(8u32)
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("summary")).text().null())
                    .col(ColumnDef::new(Alias::new("key_points")).text())
                    .col(ColumnDef::new(Alias::new("action_items")).text())
                    .col(ColumnDef::new(Alias::new("keywords")).text())
                    .col(ColumnDef::new(Alias::new("mind_map")).text())
                    .col(
                        ColumnDef::new(Alias::new("create_at"))
                            .default(Expr::current_timestamp())
                            .date_time(),
                    )
                    .to_owned(),
            )
            .await?;

        // create mind_map table
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("rss_link_mind_map"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("link_url"))
                            .string_len(32u32)
                            .primary_key()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("version"))
                            .string_len(32u32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("language"))
                            .string_len(8u32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("mind_map")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("create_at"))
                            .default(Expr::current_timestamp())
                            .date_time(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("rss_link_summary"))
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("rss_link_mind_map"))
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}
