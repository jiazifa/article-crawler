use chrono::naive::serde::ts_milliseconds_option::deserialize as from_milli_tsopt;

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};

use super::schema::{CreateOrUpdateRssLinkRequest, LinkModel, QueryRssLinkRequest};
use crate::error::ErrorInService;

use crate::DBConnection;
use chrono::NaiveDateTime;
use lib_entity::{feed_link, rss_subscription};
use lib_utils::math::{get_page_count, get_page_offset};
use sea_orm::{entity::*, query::*};
use serde::Deserialize;

pub struct LinkController;

impl LinkController {
    pub async fn insert_link(
        &self,
        req: CreateOrUpdateRssLinkRequest,
        conn: &DBConnection,
    ) -> Result<(bool, feed_link::Model), ErrorInService> {
        // 构建查找条件
        let query = match req.id.clone() {
            Some(id) => feed_link::Entity::find().filter(feed_link::Column::Id.eq(id)),
            None => feed_link::Entity::find()
                .left_join(rss_subscription::Entity)
                .filter(feed_link::Column::Link.eq(req.link.clone()))
                .filter(rss_subscription::Column::Id.eq(req.subscrption_id)),
        };
        // 执行查找
        let link = query.one(conn).await.map_err(ErrorInService::DBError)?;

        // 判断是否需要更新
        let should_update = link.is_some();
        // 如果找到了，就更新，否则就创建
        let mut new_model = match link {
            Some(m) => m.into_active_model(),
            None => feed_link::ActiveModel {
                ..Default::default()
            },
        };
        new_model.title = Set(req.title.clone());

        let image_value = serde_json::to_value(req.images.clone()).unwrap();
        let author_value = serde_json::to_value(req.authors.clone()).unwrap();
        new_model.link = Set(req.link.clone());
        new_model.description = Set(req.description.clone());
        new_model.desc_pure_txt = Set(req.desc_pure_txt.clone());
        // images is serder_json value. vec of Image
        new_model.images = Set(Some(image_value));
        new_model.authors = Set(Some(author_value));
        new_model.published_at = Set(req.published_at);

        // 执行更新或者创建
        let updated = match should_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };
        Ok((should_update, updated))
    }

    pub async fn query_links(
        &self,
        req: QueryRssLinkRequest,
        conn: &DBConnection,
    ) -> Result<PageResponse<LinkModel>, ErrorInService> {
        let mut select = req.build_query();

        let page_info = req
            .page
            .clone()
            .unwrap_or(PageRequestBuilder::default().build().unwrap());
        let page_size = page_info.page_size;
        let page = page_info.page;
        let offset = get_page_offset(page, page_size);

        // 根据时间排序, 默认是降序
        select = select
            .order_by_desc(feed_link::Column::PublishedAt)
            .limit(page_size)
            .offset(offset)
            .select();
        let all_count = select.clone().count(conn).await.unwrap_or(0);

        let page_count = get_page_count(all_count, page_size);
        let models = select.into_model().all(conn).await?;
        let resp = PageResponse::new(page_count, page, page_size, models);
        Ok(resp)
    }

    pub async fn fetch_count(
        &self,
        req: QueryRssLinkRequest,
        conn: &DBConnection,
    ) -> Result<u64, ErrorInService> {
        let select = req.build_query();
        let count = select.count(conn).await.unwrap_or(0);
        Ok(count)
    }

    pub async fn remove_expired_links(
        &self,
        expired_at: NaiveDateTime,
        conn: &DBConnection,
    ) -> Result<u64, ErrorInService> {
        let result = lib_entity::feed_link::Entity::delete_many()
            .filter(lib_entity::feed_link::Column::PublishedAt.lt(expired_at))
            .exec(conn)
            .await?;
        Ok(result.rows_affected)
    }
}

// impl builder for RssLinkReq
impl QueryRssLinkRequest {
    pub async fn fetch_count(&self, conn: &DBConnection) -> Result<u64, ErrorInService> {
        let select = self.build_query();
        let count = select.count(conn).await.unwrap_or(0);
        Ok(count)
    }

    fn build_query(&self) -> Select<feed_link::Entity> {
        let mut select = feed_link::Entity::find().inner_join(rss_subscription::Entity);
        select = select
            .select_only()
            .columns(vec![
                feed_link::Column::Id,
                feed_link::Column::Title,
                feed_link::Column::Link,
                feed_link::Column::Description,
                feed_link::Column::DescPureTxt,
                feed_link::Column::PublishedAt,
            ])
            // subscrption_id
            .column_as(rss_subscription::Column::Id, "subscrption_id")
            .column_as(feed_link::Column::Images, "images")
            // authors 是 authors_json 的解析结果
            .column_as(feed_link::Column::Authors, "authors");

        if let Some(ids) = &self.ids {
            if !ids.is_empty() {
                select = select.filter(feed_link::Column::Id.is_in(ids.clone()))
            }
        }
        if let Some(title) = &self.title {
            select = select.filter(feed_link::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(subscription_ids) = &self.subscrption_ids {
            if !subscription_ids.is_empty() {
                select = select.filter(rss_subscription::Column::Id.is_in(subscription_ids.clone()))
            }
        }

        if let Some(published_at_lower) = &self.published_at_lower {
            select = select.filter(feed_link::Column::PublishedAt.gt(*published_at_lower))
        }
        if let Some(published_at_upper) = &self.published_at_upper {
            select = select.filter(feed_link::Column::PublishedAt.lt(*published_at_upper))
        }
        select
    }
}

#[cfg(test)]
mod tests {

    use migration::{Migrator, MigratorTrait};

    use crate::rss::schema::{CreateOrUpdateRssLinkRequestBuilder, QueryRssLinkRequestBuilder};

    use super::*;

    #[tokio::test]
    async fn test_create_link() {
        let conn = crate::test_runner::setup_database().await;

        {
            let controller = LinkController;
            let current_date = chrono::Utc::now().naive_utc();

            let req = CreateOrUpdateRssLinkRequestBuilder::default()
                .title("test".to_owned())
                .link("https://www.baidu.com".to_owned())
                .subscrption_id(11)
                .images(vec![])
                .authors(vec![])
                .published_at(current_date)
                .build()
                .unwrap();
            let res = controller.insert_link(req, &conn).await.unwrap();
            assert_eq!(res.1.title, "test");

            let id = res.1.id.clone();

            let req = CreateOrUpdateRssLinkRequestBuilder::default()
                .id(id)
                .title("test_updated".to_owned())
                .link("https://www.baidu.com".to_owned())
                .subscrption_id(11)
                .images(vec![])
                .authors(vec![])
                .published_at(current_date)
                .build()
                .unwrap();
            let res = controller.insert_link(req, &conn).await.unwrap();
            assert_eq!(res.1.title, "test_updated");
        }
    }
}
