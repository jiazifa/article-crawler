use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_subscriptions_category_id_index")
                    .table(Alias::new("rss_subscriptions"))
                    .col(Alias::new("category_id"))
                    .to_owned(),
            )
            .await?;

        // create index for rss_category id
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_category_id_index")
                    .table(Alias::new("rss_category"))
                    .col(Alias::new("id"))
                    .to_owned(),
            )
            .await?;

        // create index for rss_subscriptions id
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_subscriptions_id_index")
                    .table(Alias::new("rss_subscriptions"))
                    .col(Alias::new("id"))
                    .to_owned(),
            )
            .await?;
        // create index for rss_subscriptions title
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_subscriptions_title_index")
                    .table(Alias::new("rss_subscriptions"))
                    .col(Alias::new("title"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_links_subscrption_id_index")
                    .table(Alias::new("rss_links"))
                    .col(Alias::new("subscrption_id"))
                    .to_owned(),
            )
            .await?;
        // create index for rss_links id
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_links_id_index")
                    .table(Alias::new("rss_links"))
                    .col(Alias::new("id"))
                    .to_owned(),
            )
            .await?;
        // create index for rss_links title
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("rss_links_title_index")
                    .table(Alias::new("rss_links"))
                    .col(Alias::new("title"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop index for rss_category id
        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_category"))
                    .name("rss_category_id_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_subscriptions"))
                    .name("rss_subscriptions_category_id_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        // drop index for rss_subscriptions id
        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_subscriptions"))
                    .name("rss_subscriptions_id_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        // drop index for rss_subscriptions title
        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_subscriptions"))
                    .name("rss_subscriptions_title_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_links"))
                    .name("rss_links_subscrption_id_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // drop index for rss_links id
        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_links"))
                    .name("rss_links_id_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        // drop index for rss_links title
        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("rss_links"))
                    .name("rss_links_title_index")
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
