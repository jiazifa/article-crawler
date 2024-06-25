use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 链接的 标签信息, 链接可能包含了多个标签
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    // 通过 ; 分隔多个字段
                    .add_column(ColumnDef::new(Alias::new("tags")).text().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("rss_links"))
                    .drop_column(Alias::new("tags"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
