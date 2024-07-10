use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 订阅源 更新频率配置
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("feed_build_config"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("subscription_id"))
                            .integer()
                            .primary_key()
                            .unique_key()
                            .not_null(),
                    )
                    // 定义初始假设频率
                    .col(
                        ColumnDef::new(Alias::new("initial_frequency"))
                            .float()
                            .null(),
                    )
                    // 拟合后的最佳频率
                    .col(
                        ColumnDef::new(Alias::new("fitted_frequency"))
                            .float()
                            .null(),
                    )
                    // 是否启用自适应
                    .col(
                        ColumnDef::new(Alias::new("fitted_adaptive"))
                            .boolean()
                            .null(),
                    )
                    // Source Type
                    .col(
                        ColumnDef::new(Alias::new("source_type"))
                            .string_len(32u32)
                            .null(),
                    )
                    // 最近一次更新时间 last_build_at
                    .col(
                        ColumnDef::new(Alias::new("last_build_at"))
                            .date_time()
                            .null(),
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
                    .table(Alias::new("feed_build_config"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
