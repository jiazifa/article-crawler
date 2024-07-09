use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("feed_build_record"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("identifier"))
                            .not_null()
                            .primary_key()
                            .string_len(32u32),
                    )
                    .col(
                        ColumnDef::new(Alias::new("subscription_id"))
                            .not_null()
                            .integer(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .not_null()
                            .small_integer(),
                    )
                    // remark
                    .col(
                        ColumnDef::new(Alias::new("remark"))
                            .string_len(255u32)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .default(Expr::current_timestamp())
                            .date_time(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("feed_build_record"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
