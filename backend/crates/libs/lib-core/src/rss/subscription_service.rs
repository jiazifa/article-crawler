use std::collections::{BTreeMap, HashSet};

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};
use crate::rss::link_service::LinkController;

use super::schema::{
    self, CreateOrUpdateRssLinkRequest, CreateOrUpdateRssLinkRequestBuilder,
    CreateOrUpdateSubscriptionRequest, CreateOrUpdateSubscriptionRequestBuilder,
    QueryPreferUpdateSubscriptionRequest, QueryRssLinkRequestBuilder, QuerySubscriptionRequest,
    QuerySubscriptionRequestBuilder, QuerySubscriptionsWithLinksRequest, SubscriptionModel,
    SubscriptionWithLinksResp, UpdateSubscriptionCountRequest,
};
use crate::error::ErrorInService;
use crate::DBConnection;
use chrono::{Datelike, NaiveDateTime, Timelike};
use lib_crawler::{try_get_all_image_from_html_content, try_get_all_text_from_html_content};
use lib_entity::{rss_category, rss_links, rss_subscrip_count_offset, rss_subscriptions};
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
                Ok(d) => NaiveDateTime::from_timestamp_opt(d.timestamp(), 0),
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
                        Ok(d) => NaiveDateTime::from_timestamp_opt(d.timestamp(), 0),
                        Err(_) => NaiveDateTime::from_timestamp_opt(current_timestamp, 0),
                    },
                    None => NaiveDateTime::from_timestamp_opt(current_timestamp, 0),
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
                    link_req.images_json(value);
                }
                if let Some(value) = authors_json {
                    link_req.authors_json(value);
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
    ) -> Result<(bool, String), ErrorInService> {
        let query = match req.identifier.clone() {
            Some(id) => rss_subscriptions::Entity::find()
                .filter(rss_subscriptions::Column::Identifier.eq(id)),
            None => rss_subscriptions::Entity::find()
                .filter(rss_subscriptions::Column::Link.eq(req.link.clone()))
                .filter(rss_subscriptions::Column::CategoryId.eq(req.category_id)),
        };
        let subscription = query.one(conn).await.map_err(ErrorInService::DBError)?;
        let prefer_update = subscription.is_some();
        let mut new_model = match subscription {
            Some(m) => m.into_active_model(),
            None => rss_subscriptions::ActiveModel {
                identifier: Set(uuid::Uuid::new_v4().simple().to_string()),
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
        new_model.last_build_date = Set(req.last_build_date);
        new_model.language = Set(req.language.clone());
        new_model.rating = Set(req.rating);
        new_model.visual_url = Set(req.visual_url.clone());
        new_model.sort_order = Set(req.sort_order);

        let updated = match prefer_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };
        let is_update = prefer_update;

        Ok((is_update, updated.identifier))
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

        let mut select = rss_subscriptions::Entity::find()
            .join_rev(
                JoinType::LeftJoin,
                rss_links::Entity::belongs_to(rss_subscriptions::Entity)
                    .from(rss_links::Column::SubscrptionId)
                    .to(rss_subscriptions::Column::Id)
                    .on_condition(move |_left, right| {
                        Expr::col(rss_links::Column::PublishedAt)
                            .gte(week_start_date)
                            .and(Expr::col(rss_links::Column::PublishedAt).lt(current_date))
                            .into_condition()
                    })
                    .into(),
            )
            .left_join(rss_category::Entity)
            .left_join(rss_subscrip_count_offset::Entity);
        // 查询 `SubscriptionModel` 的所有字段
        select = select
            .select_only()
            .distinct()
            .columns([
                rss_subscriptions::Column::Id,
                rss_subscriptions::Column::Identifier,
                rss_subscriptions::Column::Title,
                rss_subscriptions::Column::Description,
                rss_subscriptions::Column::Link,
                rss_subscriptions::Column::SiteLink,
                rss_subscriptions::Column::VisualUrl,
                rss_subscriptions::Column::Logo,
                rss_subscriptions::Column::Language,
                rss_subscriptions::Column::SortOrder,
                rss_subscriptions::Column::Rating,
                rss_subscriptions::Column::LastBuildDate,
            ])
            .group_by(rss_subscriptions::Column::Identifier)
            .column_as(
                Expr::col(rss_subscrip_count_offset::Column::Offset).if_null(0),
                "subscribers",
            )
            .column_as(rss_subscriptions::Column::CategoryId, "category_id")
            .column_as(Expr::cust("''"), "accent_color")
            .column_as(
                rss_subscriptions::Column::LastBuildDate.is_not_null(),
                "is_completed",
            )
            // 为 subscriber_count 字段生成一个随机数
            .column_as(Expr::cust("NULL"), "subscriber_count")
            // 为 subscribed_at 字段设置 -1
            .column_as(Expr::cust("NULL"), "subscribed_at")
            // custom_title 字段设置为空
            .column_as(Expr::cust("NULL"), "custom_title")
            // sort 字段设置为 -1
            .column_as(rss_subscriptions::Column::SortOrder, "sort_order")
            // article_count_for_this_week 查询本周文章数量
            .column_as(rss_links::Column::Id.count(), "article_count_for_this_week");

        if let Some(ids) = &req.ids {
            if !ids.is_empty() {
                // 去重后查询
                let ids_set: HashSet<i64> = ids.iter().cloned().collect();
                select = select.filter(rss_subscriptions::Column::Id.is_in(ids_set.clone()))
            }
        }
        if let Some(idfs) = &req.idfs {
            if !idfs.is_empty() {
                // 去重后查询
                let idfs_set: HashSet<String> = idfs.iter().cloned().collect();
                select =
                    select.filter(rss_subscriptions::Column::Identifier.is_in(idfs_set.clone()))
            }
        }

        if let Some(title) = &req.title {
            select = select.filter(rss_subscriptions::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(description) = &req.description {
            select = select
                .filter(rss_subscriptions::Column::Description.like(format!("%{}%", description)))
        }
        if let Some(category_id) = &req.category_id {
            select = select.filter(rss_subscriptions::Column::CategoryId.eq(category_id))
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
            .order_by_desc(rss_subscriptions::Column::UpdatedAt)
            .limit(page_size)
            .offset(offset)
            .select();

        let all_count = rss_subscriptions::Entity::find()
            .select_only()
            .column(rss_subscriptions::Column::Id)
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

    pub async fn query_subscriptions_with_links(
        &self,
        req: QuerySubscriptionsWithLinksRequest,
        conn: &DBConnection,
    ) -> Result<Vec<SubscriptionModel>, ErrorInService> {
        // 找到分类下所有订阅源，按照时间排序，取最近的N条
        let links_raw_sql = r#"
        SELECT l.id AS id
        FROM (
            SELECT s.id, c.id
            FROM rss_subscriptions s
            LEFT JOIN rss_category c ON s.category_id = c.id
            WHERE c.id = ?
        ) sc
        JOIN (
            SELECT l.id, l.subscrption_id, ROW_NUMBER() OVER (PARTITION BY l.subscrption_id ORDER BY l.created_at DESC) AS ln
            FROM rss_links l
        ) l ON sc.id = l.subscrption_id
        WHERE l.ln <= ?
        "#;
        let link_stmt = Statement::from_sql_and_values(
            conn.get_database_backend(),
            links_raw_sql,
            vec![req.category_id.into(), req.link_count.unwrap_or(1).into()],
        );
        let ori_link_models = rss_links::Entity::find()
            .select_only()
            .column_as(rss_links::Column::Id, "id")
            .from_raw_sql(link_stmt)
            .into_json()
            .all(conn)
            .await
            .map_err(|e| {
                tracing::error!("查询链接失败:{}", e);
                e
            })?;

        let link_ids: HashSet<i64> = ori_link_models
            .iter()
            .map_while(|m| match m.get("id") {
                Some(v) => v.as_i64(),
                None => None,
            })
            .collect();
        let link_ids: Vec<_> = link_ids.into_iter().collect();

        if link_ids.is_empty() {
            return Ok(Vec::new());
        }
        let link_count = link_ids.len();
        // 获得对应的链接模型
        let link_req = QueryRssLinkRequestBuilder::default()
            .ids(link_ids)
            .page(PageRequest::single_page(link_count))
            .build()
            .map_err(|_e| {
                tracing::error!("构建查询链接的请求失败:{}", _e);
                ErrorInService::Custom("构建查询链接的请求失败".to_string())
            })?;

        let link_models = LinkController
            .query_links(link_req, conn)
            .await
            .map_err(|e| {
                tracing::error!("查询链接失败:{}", e);
                e
            })?
            .data;

        // 去重
        let subscription_ids: HashSet<_> = link_models.iter().map(|m| m.subscrption_id).collect();
        let subscription_ids: Vec<_> = subscription_ids.into_iter().collect();
        let subscription_count = subscription_ids.len();

        let subscription_req = QuerySubscriptionRequestBuilder::default()
            .ids(subscription_ids)
            .page(PageRequest::single_page(subscription_count))
            .build()
            .map_err(|_e| {
                tracing::error!("构建查询订阅源的请求失败:{}", _e);
                ErrorInService::Custom("构建查询订阅源的请求失败".to_string())
            })?;

        let mut subscription_models = self.query_subscription(subscription_req, conn).await?.data;

        tracing::info!(
            "link_models_count: {:?} / subscription_models_count: {:?}",
            link_models.len(),
            subscription_models.len()
        );
        let result = subscription_models
            .iter_mut()
            .map(|m| {
                let mut links = Vec::new();
                link_models.iter().for_each(|l| {
                    let mut l_clone = l.clone();
                    // 手动清理掉content字段
                    l_clone.content = None;
                    if l.subscrption_id == m.id {
                        links.push(l_clone);
                    }
                });
                schema::SubscriptionModel {
                    links: Some(links),
                    ..m.clone()
                }
            })
            .collect();
        Ok(result)
    }

    pub async fn update_subscriptor_count(
        &self,
        req: UpdateSubscriptionCountRequest,
        conn: &DBConnection,
    ) -> Result<(), ErrorInService> {
        let subscription = rss_subscrip_count_offset::Entity::find()
            .filter(rss_subscrip_count_offset::Column::SubscriptionId.eq(req.subscription_id))
            .one(conn)
            .await
            .map_err(ErrorInService::DBError)?;
        let temp_offset = match subscription.clone() {
            Some(m) => m.offset + req.offset,
            None => req.offset,
        };
        match subscription {
            Some(m) => {
                let mut new_model = m.into_active_model();
                new_model.offset = Set(temp_offset);
                new_model.update(conn).await?;
            }
            None => {
                let new_model = rss_subscrip_count_offset::ActiveModel {
                    subscription_id: Set(req.subscription_id),
                    offset: Set(temp_offset),
                };
                new_model.insert(conn).await?;
            }
        };

        Ok(())
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
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let conn = crate::get_db_conn(base_url).await;
        Migrator::up(&conn, None).await.unwrap();
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
        let (is_update, identifier) = controller.insert_subscription(req, &conn).await.unwrap();
        assert!(!is_update);
        assert!(!identifier.is_empty());

        let req = CreateOrUpdateSubscriptionRequestBuilder::default()
            .identifier(identifier)
            .title("updated".to_string())
            .description("updated".to_string())
            .category_id(category.id)
            .link("updated".to_string())
            .site_link("updated".to_string())
            .build()
            .unwrap();
        let (is_update, identifier) = controller.insert_subscription(req, &conn).await.unwrap();
        assert!(is_update);
        assert!(!identifier.is_empty());

        // 测试查询订阅源
        let req = QuerySubscriptionRequestBuilder::default()
            .idfs(vec![identifier.clone()])
            .build()
            .unwrap();
        let res = controller.query_subscription(req, &conn).await.unwrap();
        assert_eq!(res.data.len(), 1);
    }

    #[tokio::test]
    // test subscription offset
    async fn test_subscription_offset() {
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let conn = crate::get_db_conn(base_url).await;
        Migrator::up(&conn, None).await.unwrap();

        let offset = rss_subscrip_count_offset::ActiveModel {
            subscription_id: Set(11),
            offset: Set(1),
        };
        let updated = offset.insert(&conn).await.unwrap();
        assert_eq!(updated.subscription_id, 11);
    }
}
