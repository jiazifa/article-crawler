use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        create_table(manager).await?;
        create_index(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        drop_table(manager).await
    }
}

/// 创建表格
async fn create_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_category_table(manager).await?;

    create_subscription_table(manager).await?;

    create_links_table(manager).await?;

    Ok(())
}

async fn create_links_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Alias::new("rss_links"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .not_null()
                        .primary_key()
                        .auto_increment()
                        .integer(),
                )
                .col(
                    ColumnDef::new(Alias::new("identifier"))
                        .string_len(32u32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("title"))
                        .string_len(255u32)
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("subscrption_id"))
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(Alias::new("description")).text().null())
                .col(ColumnDef::new(Alias::new("link")).text().not_null())
                .col(
                    ColumnDef::new(Alias::new("published_at"))
                        .date_time()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .default(Expr::current_timestamp())
                        .date_time(),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .default(Expr::current_timestamp())
                        .date_time(),
                )
                .to_owned(),
        )
        .await
}

async fn create_subscription_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Alias::new("rss_subscriptions"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .not_null()
                        .primary_key()
                        .auto_increment()
                        .integer(),
                )
                .col(
                    ColumnDef::new(Alias::new("identifier"))
                        .string_len(32u32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("title"))
                        .string_len(255u32)
                        .null(),
                )
                .col(ColumnDef::new(Alias::new("description")).text().null())
                .col(ColumnDef::new(Alias::new("link")).text().null())
                .col(ColumnDef::new(Alias::new("category_id")).integer().null())
                .col(
                    ColumnDef::new(Alias::new("site_link"))
                        .string_len(255u32)
                        .null(),
                )
                .col(ColumnDef::new(Alias::new("logo")).text().null())
                .col(
                    ColumnDef::new(Alias::new("language"))
                        .string_len(64u32)
                        .null(),
                )
                .col(ColumnDef::new(Alias::new("rating")).integer().null())
                .col(ColumnDef::new(Alias::new("visual_url")).text().null())
                .col(ColumnDef::new(Alias::new("sort_order")).integer().null())
                .col(ColumnDef::new(Alias::new("pub_date")).date_time().null())
                .col(
                    ColumnDef::new(Alias::new("last_build_date"))
                        .date_time()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .default(Expr::current_timestamp())
                        .date_time(),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .default(Expr::current_timestamp())
                        .date_time(),
                )
                .to_owned(),
        )
        .await
}

async fn create_category_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Alias::new("rss_category"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .primary_key()
                        .auto_increment()
                        .not_null(),
                )
                .col(ColumnDef::new(Alias::new("identifier")).string_len(32u32))
                .col(ColumnDef::new(Alias::new("title")).string_len(32u32))
                .col(ColumnDef::new(Alias::new("description")).text())
                .col(ColumnDef::new(Alias::new("parent_id")).integer())
                .col(ColumnDef::new(Alias::new("icon")).string_len(32u32))
                .col(ColumnDef::new(Alias::new("sort_order")).integer())
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .date_time()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .date_time()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await
}

//  创建索引
async fn create_index(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_category_identifier_index")
            .table(Alias::new("rss_category"))
            .col(Alias::new("identifier"))
            .unique()
            .to_owned(),
    )
    .await?;

    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_subscription_identifier_index")
            .table(Alias::new("rss_subscriptions"))
            .col(Alias::new("identifier"))
            .unique()
            .to_owned(),
    )
    .await?;

    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_identifier_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("identifier"))
            .unique()
            .to_owned(),
    )
    .await
}

// 删除表格
async fn drop_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    println!("开始删除表格----------");
    manager
        .drop_table(Table::drop().table(Alias::new("rss_links")).to_owned())
        .await?;
    manager
        .drop_table(
            Table::drop()
                .table(Alias::new("rss_subscriptions"))
                .to_owned(),
        )
        .await?;
    manager
        .drop_table(Table::drop().table(Alias::new("rss_category")).to_owned())
        .await?;

    Ok(())
}
