use std::collections::{BTreeMap, HashSet};

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};
use crate::feed::link_service::LinkController;

use super::schema::{
    self, Author, CreateOrUpdateRssLinkRequest, CreateOrUpdateRssLinkRequestBuilder,
    CreateOrUpdateSubscriptionRequest, CreateOrUpdateSubscriptionRequestBuilder, Image,
    QueryPreferUpdateSubscriptionRequest, QueryRssLinkRequestBuilder, QuerySubscriptionRequest,
    QuerySubscriptionRequestBuilder, QuerySubscriptionsWithLinksRequest, SubscriptionModel,
    SubscriptionWithLinksResp, UpdateSubscriptionCountRequest,
};
use crate::error::ErrorInService;
use crate::{auth, DBConnection};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike};
use lib_crawler::{try_get_all_image_from_html_content, try_get_all_text_from_html_content};
use lib_entity::{feed_build_config, feed_category, feed_link, feed_subscription};
use lib_utils::math::{get_page_count, get_page_offset};
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::DbBackend;
use sea_orm::{entity::*, query::*};
use serde::Deserialize;
use tokio::sync::TryAcquireError;

pub struct SubscriptionController;

impl SubscriptionController {
    pub async fn insert_subscription(
        &self,
        req: CreateOrUpdateSubscriptionRequest,
        conn: &DBConnection,
    ) -> Result<(bool, i64), ErrorInService> {
        let query = match req.id.clone() {
            Some(id) => {
                feed_subscription::Entity::find().filter(feed_subscription::Column::Id.eq(id))
            }
            None => feed_subscription::Entity::find()
                // 通过 `rss_subscriptions_category` 表查询关联的 `category_id`
                .left_join(feed_category::Entity)
                .filter(feed_subscription::Column::Link.eq(req.link.clone()))
                .filter(feed_category::Column::Id.eq(req.category_id)),
        };

        let subscription = query.one(conn).await.map_err(ErrorInService::DBError)?;

        let prefer_update = subscription.is_some();
        // 如果 prefer_update 为 false, 并且 req.category_id 为 None, 则返回错误
        if !prefer_update && req.category_id.is_none() {
            return Err(ErrorInService::Custom(
                "category_id is required".to_string(),
            ));
        }
        let mut new_model = match subscription {
            Some(m) => m.into_active_model(),
            None => feed_subscription::ActiveModel {
                ..Default::default()
            },
        };
        new_model.title = Set(req.title.clone());
        new_model.description = Set(req.description.clone());
        new_model.link = Set(Some(req.link.clone()));
        new_model.site_link = Set(req.site_link.clone());
        new_model.category_id = Set(req.category_id);
        new_model.logo = Set(req.logo.clone());
        new_model.pub_date = Set(req.pub_date);
        new_model.language = Set(req.language.clone());
        new_model.rating = Set(req.rating);
        new_model.visual_url = Set(req.visual_url.clone());
        new_model.sort_order = Set(req.sort_order);

        let updated = match prefer_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };

        let is_update = prefer_update;
        if !is_update {
            let build_config = feed_build_config::ActiveModel {
                subscription_id: Set(updated.id),
                initial_frequency: Set(3600.0),
                source_type: Set(feed_build_config::SourceType::Unknown),
                ..Default::default()
            };
            if let Err(e) = build_config.insert(conn).await {
                tracing::error!("insert build config error: {:?}", e);
            }
        }

        Ok((is_update, updated.id))
    }

    pub async fn query_subscription(
        &self,
        req: QuerySubscriptionRequest,
        conn: &DBConnection,
    ) -> Result<PageResponse<schema::SubscriptionModel>, ErrorInService> {
        let current_date = chrono::Utc::now().naive_utc();
        // get start of this week, datetime is 00:00:00
        let week_start_date = current_date
            .checked_sub_signed(chrono::Duration::days(
                current_date.weekday().num_days_from_monday() as i64,
            ))
            .unwrap_or(current_date)
            .with_hour(0)
            .unwrap_or(current_date)
            .with_minute(0)
            .unwrap_or(current_date)
            .with_second(0);

        let mut select = feed_subscription::Entity::find()
            .left_join(feed_category::Entity)
            .left_join(feed_build_config::Entity)
            .join(
                JoinType::LeftJoin,
                lib_entity::feed_subscription::Relation::Links
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col(lib_entity::feed_link::Column::PublishedAt)
                            .gte(week_start_date)
                            .and(
                                Expr::col(lib_entity::feed_link::Column::PublishedAt)
                                    .lt(current_date),
                            )
                            .into_condition()
                    })
                    .into(),
            );

        // 查询 `SubscriptionModel` 的所有字段
        select = select
            .select_only()
            .distinct()
            .columns([
                feed_subscription::Column::Id,
                feed_subscription::Column::Title,
                feed_subscription::Column::Description,
                feed_subscription::Column::Link,
                feed_subscription::Column::SiteLink,
                feed_subscription::Column::VisualUrl,
                feed_subscription::Column::Logo,
                feed_subscription::Column::Language,
                feed_subscription::Column::SortOrder,
                feed_subscription::Column::Rating,
            ])
            .group_by(feed_subscription::Column::Id)
            .column_as(Expr::cust("''"), "accent_color")
            .column_as(
                feed_build_config::Column::LastBuildAt.is_not_null(),
                "is_completed",
            )
            // sort 字段设置为 -1
            .column_as(feed_subscription::Column::SortOrder, "sort_order")
            .column_as(feed_category::Column::Id, "category_id")
            // article_count_for_this_week 查询本周文章数量
            .column_as(feed_link::Column::Id.count(), "article_count_for_this_week");

        if let Some(ids) = &req.ids {
            if !ids.is_empty() {
                // 去重后查询
                let ids_set: HashSet<i64> = ids.iter().cloned().collect();
                select = select.filter(feed_subscription::Column::Id.is_in(ids_set.clone()))
            }
        }

        if let Some(title) = &req.title {
            select = select.filter(feed_subscription::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(category_id) = &req.category_id {
            select = select.filter(feed_category::Column::Id.eq(*category_id))
        }
        let page_info = req
            .page
            .clone()
            .unwrap_or(PageRequestBuilder::default().build().unwrap());
        let page_size = page_info.page_size;
        let page = page_info.page;
        let offset = get_page_offset(page, page_size);

        // 根据时间排序, 默认是降序
        select = select
            .order_by_desc(feed_subscription::Column::UpdatedAt)
            .limit(page_size)
            .offset(offset)
            .select();

        let all_count = feed_subscription::Entity::find()
            .select_only()
            .column(feed_subscription::Column::Id)
            .count(conn)
            .await
            .unwrap_or(0);
        let page_count = get_page_count(all_count, page_size);

        let models = select
            .into_model::<schema::SubscriptionModel>()
            .all(conn)
            .await?;
        let resp = PageResponse::new(page_count, page, page_size, models);
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {

    use crate::feed::{
        category_service::CategoryController, subscription_parse::SubscriptionParseController,
    };

    use super::*;
    use migration::{Migrator, MigratorTrait};

    #[tokio::test]
    async fn test_parser_rss_from_url() {
        let url: &str = "https://www.elconfidencialdigital.com/rss?seccion=el_confidencial_digital";
        let resp = SubscriptionParseController::parser_rss_from_url(url)
            .await
            .unwrap();
        // assert_eq!(resp.subscription.title, "数字尾巴");
        assert!(!resp.links.is_empty());
    }

    #[tokio::test]
    async fn test_create_subscription() {
        let conn = crate::test_runner::setup_database().await;

        let current_time: NaiveDateTime = chrono::Utc::now().naive_utc();
        let category_controller = CategoryController;
        let mut category_req_builder =
            crate::feed::schema::CreateOrUpdateCategoryRequestBuilder::default();
        category_req_builder.title("test_category".to_string());
        let category_req = category_req_builder.build().unwrap();
        let category = category_controller
            .insert_category(category_req, &conn)
            .await
            .unwrap();

        let controller = SubscriptionController;
        let req = CreateOrUpdateSubscriptionRequestBuilder::default()
            .title("test".to_string())
            .description("test".to_string())
            .link("test".to_string())
            .category_id(category.id)
            .site_link("test".to_string())
            .pub_date(current_time)
            .language("test".to_string())
            .build()
            .unwrap();
        let (is_update, id) = controller
            .insert_subscription(req.clone(), &conn)
            .await
            .unwrap();
        assert_eq!(is_update, false);

        let (is_update, id) = controller.insert_subscription(req, &conn).await.unwrap();
        assert_eq!(is_update, true);

        let req = CreateOrUpdateSubscriptionRequestBuilder::default()
            .title("updated".to_string())
            .description("updated".to_string())
            .category_id(category.id)
            .link("updated".to_string())
            .site_link("updated".to_string())
            .build()
            .unwrap();
        let (is_update, id) = controller.insert_subscription(req, &conn).await.unwrap();
        assert_eq!(is_update, false);

        let current_date = chrono::Utc::now().naive_utc();
        // get start of this week, datetime is 00:00:00
        let week_start_date = current_date
            .checked_sub_signed(chrono::Duration::days(
                current_date.weekday().num_days_from_monday() as i64,
            ))
            .unwrap_or(current_date)
            .with_hour(0)
            .unwrap_or(current_date)
            .with_minute(0)
            .unwrap_or(current_date)
            .with_second(0);

        // simple query
        let mut select = lib_entity::feed_subscription::Entity::find()
            .join(
                JoinType::LeftJoin,
                lib_entity::feed_subscription::Relation::Links
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col(lib_entity::feed_link::Column::PublishedAt)
                            .gte(week_start_date)
                            .and(
                                Expr::col(lib_entity::feed_link::Column::PublishedAt)
                                    .lt(current_date),
                            )
                            .into_condition()
                    })
                    .into(),
            )
            .left_join(lib_entity::feed_category::Entity)
            .left_join(lib_entity::feed_build_config::Entity)
            .filter(lib_entity::feed_subscription::Column::Id.eq(id));
        select = select
            .distinct()
            .select_only()
            .columns([
                feed_subscription::Column::Id,
                feed_subscription::Column::Title,
                feed_subscription::Column::Description,
                feed_subscription::Column::Link,
                feed_subscription::Column::SiteLink,
                feed_subscription::Column::VisualUrl,
                feed_subscription::Column::Logo,
                feed_subscription::Column::Language,
                feed_subscription::Column::SortOrder,
                feed_subscription::Column::Rating,
                feed_subscription::Column::CreatedAt,
                feed_subscription::Column::UpdatedAt,
            ])
            .column_as(feed_category::Column::Id, "category_id")
            .column_as(
                feed_build_config::Column::LastBuildAt.is_not_null(),
                "is_completed",
            )
            // sort 字段设置为 -1
            .column_as(feed_subscription::Column::SortOrder, "sort_order")
            // article_count_for_this_week 查询本周文章数量
            .column_as(feed_link::Column::Id.count(), "article_count_for_this_week")
            .group_by(feed_subscription::Column::Id);

        // .column_as(rss_link::Column::Id.count(), "article_count_for_this_week");

        println!("sql: {}", select.build(conn.get_database_backend()));
        let models = select
            .into_model::<crate::feed::schema::SubscriptionModel>()
            .all(&conn)
            .await
            .unwrap();

        assert_eq!(models.len(), 1);
    }
}
