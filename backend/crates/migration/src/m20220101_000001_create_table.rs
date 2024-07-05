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

    create_middle_table(manager).await?;

    Ok(())
}

// - MARK: 创建表格
/// 创建rss_links表格
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
                    ColumnDef::new(Alias::new("title"))
                        .string_len(255u32)
                        .null(),
                )
                .col(ColumnDef::new(Alias::new("description")).text().null())
                // desc_pure_text
                .col(ColumnDef::new(Alias::new("desc_pure_txt")).text().null())
                // images json
                .col(ColumnDef::new(Alias::new("images")).json_binary().null())
                // authors json
                .col(ColumnDef::new(Alias::new("authors")).json_binary().null())
                // link
                .col(ColumnDef::new(Alias::new("link")).text().not_null())
                // tags 一个文章可能有多个标签
                .col(
                    ColumnDef::new(Alias::new("tags"))
                        .array(ColumnType::String(Some(64u32)))
                        .null(),
                )
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
                    ColumnDef::new(Alias::new("title"))
                        .string_len(255u32)
                        .null(),
                )
                .col(ColumnDef::new(Alias::new("description")).text().null())
                .col(ColumnDef::new(Alias::new("link")).text().null())
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

// 创建中间表，用于关联 rss_links 和 rss_subscriptions, rss_subscription 和 rss_category
async fn create_middle_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    // 创建 rss_links 和 rss_subscriptions 中间表, 通过 subscrption_id 关联
    manager
        .create_table(
            Table::create()
                .table(Alias::new("rss_links_subscriptions"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .primary_key()
                        .auto_increment()
                        .not_null(),
                )
                .foreign_key(
                    &mut ForeignKey::create()
                        .name("rss_links_subscriptions_link_id_fk")
                        .from(Alias::new("rss_links"), Alias::new("id"))
                        .to_col(Alias::new("link_id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    &mut ForeignKey::create()
                        .name("rss_links_subscriptions_subscrption_id_fk")
                        .from(Alias::new("rss_subscriptions"), Alias::new("id"))
                        .to_col(Alias::new("subscrption_id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    // 创建 rss_subscriptions 和 rss_category 中间表, 通过 category_id 关联
    manager
        .create_table(
            Table::create()
                .table(Alias::new("rss_subscriptions_category"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .primary_key()
                        .auto_increment()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("subscrption_id"))
                        .integer()
                        .not_null()
                        .comment("rss_subscriptions id"),
                )
                .col(
                    ColumnDef::new(Alias::new("category_id"))
                        .integer()
                        .not_null()
                        .comment("rss_category id"),
                )
                .to_owned(),
        )
        .await?;

    Ok(())
}

// - MARK: 创建索引
//  创建索引
async fn create_index(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    // 创建 rss_subscription 和 rss_category 索引

    // create index for rss_category id
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_category_id_index")
            .table(Alias::new("rss_category"))
            .col(Alias::new("id"))
            .to_owned(),
    )
    .await?;

    // create index for rss_category title
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_category_title_index")
            .table(Alias::new("rss_category"))
            .col(Alias::new("title"))
            .to_owned(),
    )
    .await?;

    // create index for rss_subscriptions id
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_subscriptions_id_index")
            .table(Alias::new("rss_subscriptions"))
            .col(Alias::new("id"))
            .to_owned(),
    )
    .await?;
    // create index for rss_subscriptions title
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_subscriptions_title_index")
            .table(Alias::new("rss_subscriptions"))
            .col(Alias::new("title"))
            .to_owned(),
    )
    .await?;
    // create index for rss_subscription link
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_subscriptions_link_index")
            .table(Alias::new("rss_subscriptions"))
            .col(Alias::new("link"))
            .to_owned(),
    )
    .await?;

    // create index for rss_links id
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_id_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("id"))
            .to_owned(),
    )
    .await?;
    // create index for rss_links title
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_title_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("title"))
            .to_owned(),
    )
    .await?;
    // desc_pure_txt & description
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_desc_pure_txt_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("desc_pure_txt"))
            .to_owned(),
    )
    .await?;
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_description_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("description"))
            .to_owned(),
    )
    .await?;

    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_link_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("link"))
            .to_owned(),
    )
    .await?;

    // index for images / authors / tags
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_images_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("images"))
            .to_owned(),
    )
    .await?;

    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_authors_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("authors"))
            .to_owned(),
    )
    .await?;

    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_tags_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("tags"))
            .to_owned(),
    )
    .await?;

    // index for published_at
    m.create_index(
        Index::create()
            .if_not_exists()
            .name("rss_links_published_at_index")
            .table(Alias::new("rss_links"))
            .col(Alias::new("published_at"))
            .to_owned(),
    )
    .await?;

    Ok(())
}

// - MARK: 删除表格
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
    // 移除中间表
    manager
        .drop_table(
            Table::drop()
                .table(Alias::new("rss_links_subscriptions"))
                .to_owned(),
        )
        .await?;
    manager
        .drop_table(
            Table::drop()
                .table(Alias::new("rss_subscriptions_category"))
                .to_owned(),
        )
        .await?;
    Ok(())
}
