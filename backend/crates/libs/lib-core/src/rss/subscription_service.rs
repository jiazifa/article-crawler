use std::collections::{BTreeMap, HashSet};

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};
use crate::rss::link_service::LinkController;

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
use lib_entity::{
    rss_category, rss_link, rss_subscription, rss_subscription_category, rss_subscription_config,
};
use lib_utils::math::{get_page_count, get_page_offset};
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::DbBackend;
use sea_orm::{entity::*, query::*};
use serde::Deserialize;
use tokio::sync::TryAcquireError;

pub struct SubscriptionController;

impl SubscriptionController {
    // 从url中解析订阅源
    pub async fn parser_rss_from_url<T: AsRef<str>>(
        &self,
        url: T,
    ) -> Result<SubscriptionWithLinksResp, ErrorInService> {
        let rss = lib_crawler::fetch_rss_from_url(url.as_ref())
            .await
            .map_err(|e| ErrorInService::Custom(format!("解析RSS失败:{}", e)))?;
        let mut links: Vec<CreateOrUpdateRssLinkRequest> = Vec::new();
        let pub_date = match rss.pub_date() {
            Some(d) => match dateparser::parse(d) {
                Ok(d) => Some(d.to_utc().naive_utc()),
                Err(_) => None,
            },
            None => None,
        };
        let language = rss.language().map(|l| l.to_string());

        let mut subscription_req = CreateOrUpdateSubscriptionRequestBuilder::default();
        subscription_req.title(rss.title().to_string());

        subscription_req.description(rss.description().to_string());
        subscription_req.link(url.as_ref().to_string());
        subscription_req.site_link(rss.link().to_string());
        if let Some(value) = pub_date {
            subscription_req.pub_date(value);
        }
        if let Some(value) = language {
            subscription_req.language(value);
        }
        let subscription = subscription_req.clone().build()?;

        let current_timestamp = chrono::Utc::now().timestamp();
        rss.items().iter().for_each(|item| {
            if let Some(item_link) = item.link() {
                let pub_date = match item.pub_date() {
                    Some(d) => match dateparser::parse(d) {
                        Ok(d) => Some(d.naive_utc()), // 包装在Some中
                        Err(_) => DateTime::from_timestamp(current_timestamp, 0)
                            .and_then(|d| Some(d.to_utc().naive_utc())),
                        // 修改这里，确保返回Option
                    },
                    None => DateTime::from_timestamp(current_timestamp, 0)
                        .and_then(|d| Some(d.to_utc().naive_utc())),
                };
                let ext = item.extensions();
                let mut ext_map: BTreeMap<String, Vec<rss::extension::Extension>> = BTreeMap::new();
                if let Some(e_map) = ext.get("ext") {
                    ext_map = e_map.to_owned();
                }
                let authors_json = match ext_map.get("authors") {
                    Some(authors) => {
                        let mut authors_j = Vec::new();
                        // insert from extension's attr
                        authors.iter().for_each(|author| {
                            let attr = author.attrs();
                            authors_j.push(attr);
                        });
                        serde_json::to_string(&authors_j).ok()
                    }
                    None => None,
                };
                let mut images_json = match ext_map.get("images") {
                    Some(images) => {
                        let mut urls = Vec::new();
                        images.iter().for_each(|image| {
                            if let Some(url) = image.value() {
                                urls.push(url.to_string());
                            }
                        });
                        serde_json::to_string(&urls).ok()
                    }
                    None => None,
                };

                if let (true, Some(desc)) = (images_json.is_none(), item.description()) {
                    if let Ok(images) = try_get_all_image_from_html_content(desc.to_string()) {
                        let images = images
                            .iter()
                            .map(|i| rss::Image {
                                url: i.to_string(),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();
                        images_json = serde_json::to_string(&images).ok();
                    }
                }
                // 解析纯文本
                let mut pure_desc: Option<String> = None;

                if let Some(desc) = item.description() {
                    if let Ok(desc) = try_get_all_text_from_html_content(desc.to_string()) {
                        pure_desc = Some(desc);
                    }
                }
                let mut link_req = CreateOrUpdateRssLinkRequestBuilder::default();

                link_req.title(match item.title() {
                    Some(t) => t.to_string(),
                    None => "".to_string(),
                });
                link_req.subscrption_id(0);
                link_req.link(item_link.to_string());
                if let Some(value) = item.description.clone() {
                    link_req.description(value);
                }

                if let Some(value) = pure_desc {
                    link_req.desc_pure_txt(value);
                }

                if let Some(value) = images_json {
                    let images: Vec<Image> =
                        serde_json::from_str::<Vec<Image>>(&value).unwrap_or_default();
                    link_req.images(images.iter().map(|i| i.clone()).collect::<Vec<_>>());
                }
                if let Some(value) = authors_json {
                    let authors = serde_json::from_str::<Vec<Author>>(&value).unwrap_or_default();
                    link_req.authors(authors.iter().map(|a| a.clone()).collect::<Vec<_>>());
                }
                if let Some(value) = pub_date {
                    link_req.published_at(value);
                }
                if let Ok(link) = link_req
                    .build()
                    .map_err(|e| ErrorInService::Custom(format!("构建链接失败:{}", e)))
                {
                    links.push(link);
                }
            }
        });
        let resp = SubscriptionWithLinksResp {
            subscription,
            links,
        };
        Ok(resp)
    }

    pub async fn insert_subscription(
        &self,
        req: CreateOrUpdateSubscriptionRequest,
        conn: &DBConnection,
    ) -> Result<(bool, i64), ErrorInService> {
        let query = match req.id.clone() {
            Some(id) => {
                rss_subscription::Entity::find().filter(rss_subscription::Column::Id.eq(id))
            }
            None => rss_subscription::Entity::find()
                // 通过 `rss_subscriptions_category` 表查询关联的 `category_id`
                .left_join(rss_category::Entity)
                .filter(rss_subscription::Column::Link.eq(req.link.clone()))
                .filter(rss_category::Column::Id.eq(req.category_id)),
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
            None => rss_subscription::ActiveModel {
                ..Default::default()
            },
        };
        new_model.title = Set(req.title.clone());
        new_model.description = Set(req.description.clone());
        new_model.link = Set(Some(req.link.clone()));
        new_model.site_link = Set(req.site_link.clone());
        new_model.logo = Set(req.logo.clone());
        new_model.pub_date = Set(req.pub_date);
        new_model.language = Set(req.language.clone());
        new_model.rating = Set(req.rating);
        new_model.visual_url = Set(req.visual_url.clone());
        new_model.sort_order = Set(req.sort_order);

        let updated = match prefer_update {
            true => new_model.update(conn).await?,
            false => {
                let m = new_model.insert(conn).await?;
                // 使用更简洁的方式处理 Option 类型
                let category_id = req.category_id.unwrap_or(0); // 假设 0 是一个合理的默认值，否则应该使用更合适的错误处理
                let relation_data = rss_subscription_category::ActiveModel {
                    category_id: Set(category_id),
                    subscription_id: Set(m.id.clone()),
                    ..Default::default()
                };
                relation_data.insert(conn).await?;
                m
            }
        };
        let is_update = prefer_update;

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

        let mut select = rss_subscription::Entity::find()
            .left_join(rss_category::Entity)
            .left_join(rss_subscription_config::Entity)
            .join(
                JoinType::LeftJoin,
                lib_entity::rss_subscription::Relation::Links.def(),
            )
            .join(
                JoinType::LeftJoin,
                lib_entity::rss_subscription_link::Relation::Link
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col(lib_entity::rss_link::Column::PublishedAt)
                            .gte(week_start_date)
                            .and(
                                Expr::col(lib_entity::rss_link::Column::PublishedAt)
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
                rss_subscription::Column::Id,
                rss_subscription::Column::Title,
                rss_subscription::Column::Description,
                rss_subscription::Column::Link,
                rss_subscription::Column::SiteLink,
                rss_subscription::Column::VisualUrl,
                rss_subscription::Column::Logo,
                rss_subscription::Column::Language,
                rss_subscription::Column::SortOrder,
                rss_subscription::Column::Rating,
            ])
            .group_by(rss_subscription::Column::Id)
            .column_as(Expr::cust("''"), "accent_color")
            .column_as(
                rss_subscription_config::Column::LastBuildAt.is_not_null(),
                "is_completed",
            )
            // sort 字段设置为 -1
            .column_as(rss_subscription::Column::SortOrder, "sort_order")
            .column_as(rss_category::Column::Id, "category_id")
            // article_count_for_this_week 查询本周文章数量
            .column_as(rss_link::Column::Id.count(), "article_count_for_this_week");

        if let Some(ids) = &req.ids {
            if !ids.is_empty() {
                // 去重后查询
                let ids_set: HashSet<i64> = ids.iter().cloned().collect();
                select = select.filter(rss_subscription::Column::Id.is_in(ids_set.clone()))
            }
        }

        if let Some(title) = &req.title {
            select = select.filter(rss_subscription::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(category_id) = &req.category_id {
            select = select.filter(rss_category::Column::Id.eq(*category_id))
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
            .order_by_desc(rss_subscription::Column::UpdatedAt)
            .limit(page_size)
            .offset(offset)
            .select();

        let all_count = rss_subscription::Entity::find()
            .select_only()
            .column(rss_subscription::Column::Id)
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

    use crate::rss::category_service::CategoryController;

    use super::*;
    use migration::{Migrator, MigratorTrait};

    #[tokio::test]
    async fn test_parser_rss_from_url() {
        let url: &str = "https://www.elconfidencialdigital.com/rss?seccion=el_confidencial_digital";
        let resp = SubscriptionController
            .parser_rss_from_url(url)
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
            crate::rss::schema::CreateOrUpdateCategoryRequestBuilder::default();
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
        let (is_update, id) = controller.insert_subscription(req, &conn).await.unwrap();
        assert_eq!(is_update, false);

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
        let mut select = lib_entity::rss_subscription::Entity::find()
            .join(
                JoinType::LeftJoin,
                lib_entity::rss_subscription::Relation::Links.def(),
            )
            .join(
                JoinType::LeftJoin,
                lib_entity::rss_subscription_link::Relation::Link
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col(lib_entity::rss_link::Column::PublishedAt)
                            .gte(week_start_date)
                            .and(
                                Expr::col(lib_entity::rss_link::Column::PublishedAt)
                                    .lt(current_date),
                            )
                            .into_condition()
                    })
                    .into(),
            )
            .left_join(lib_entity::rss_category::Entity)
            .left_join(lib_entity::rss_subscription_config::Entity)
            .filter(lib_entity::rss_subscription::Column::Id.eq(id));
        select = select
            .distinct()
            .select_only()
            .columns([
                rss_subscription::Column::Id,
                rss_subscription::Column::Title,
                rss_subscription::Column::Description,
                rss_subscription::Column::Link,
                rss_subscription::Column::SiteLink,
                rss_subscription::Column::VisualUrl,
                rss_subscription::Column::Logo,
                rss_subscription::Column::Language,
                rss_subscription::Column::SortOrder,
                rss_subscription::Column::Rating,
                rss_subscription::Column::CreatedAt,
                rss_subscription::Column::UpdatedAt,
            ])
            .column_as(rss_category::Column::Id, "category_id")
            .column_as(
                rss_subscription_config::Column::LastBuildAt.is_not_null(),
                "is_completed",
            )
            // sort 字段设置为 -1
            .column_as(rss_subscription::Column::SortOrder, "sort_order")
            // article_count_for_this_week 查询本周文章数量
            .column_as(rss_link::Column::Id.count(), "article_count_for_this_week")
            .group_by(rss_subscription::Column::Id);

        // .column_as(rss_link::Column::Id.count(), "article_count_for_this_week");

        println!("sql: {}", select.build(conn.get_database_backend()));
        let models = select
            .into_model::<crate::rss::schema::SubscriptionModel>()
            .all(&conn)
            .await
            .unwrap();

        assert_eq!(models.len(), 1);
    }
}
