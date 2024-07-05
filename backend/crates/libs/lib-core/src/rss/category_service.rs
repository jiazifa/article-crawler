use std::collections::HashMap;

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};

use super::schema::{self, CreateOrUpdateCategoryRequest, QueryCategoryRequest};
use crate::error::ErrorInService;

use crate::DBConnection;
use lib_entity::rss_category;
use lib_utils::math::{get_page_count, get_page_offset};
use sea_orm::{entity::*, query::*};
use serde::Deserialize;

pub struct CategoryController;

impl CategoryController {
    pub async fn insert_category(
        &self,
        req: CreateOrUpdateCategoryRequest,
        conn: &DBConnection,
    ) -> Result<schema::CategoryModel, ErrorInService> {
        let query = match req.id.clone() {
            Some(idf) => rss_category::Entity::find().filter(rss_category::Column::Id.eq(idf)),
            None => rss_category::Entity::find()
                .filter(rss_category::Column::Title.eq(req.title.clone())),
        };

        let category = query.one(conn).await.map_err(ErrorInService::DBError)?;
        let prefer_update = category.is_some();
        let mut new_model = match category {
            Some(m) => m.into_active_model(),
            None => rss_category::ActiveModel {
                ..Default::default()
            },
        };
        new_model.parent_id = Set(req.parent_id);
        new_model.title = Set(req.title.clone());
        new_model.description = Set(req.description.clone());
        new_model.sort_order = Set(req.sort_order);
        let updated = match prefer_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };
        Ok(updated.into())
    }

    pub async fn query_category<C>(
        &self,
        req: QueryCategoryRequest,
        conn: &C,
    ) -> Result<Vec<schema::CategoryModel>, ErrorInService>
    where
        C: ConnectionTrait,
    {
        let mut select = rss_category::Entity::find();
        if let Some(ids) = &req.ids {
            if !ids.is_empty() {
                select = select.filter(rss_category::Column::Id.is_in(ids.clone()))
            }
        }
        if let Some(title) = &req.title {
            select = select.filter(rss_category::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(description) = &req.description {
            select =
                select.filter(rss_category::Column::Description.like(format!("%{}%", description)))
        }
        if let Some(parent_ids) = &req.parent_ids {
            if !parent_ids.is_empty() {
                select = select.filter(rss_category::Column::ParentId.is_in(parent_ids.clone()))
            }
        }

        // 根据Sort order 和时间 排序
        select = select.order_by_desc(rss_category::Column::SortOrder);
        select = select.order_by_desc(rss_category::Column::UpdatedAt);

        let mut models = select
            .into_model()
            .all(conn)
            .await
            .map_err(ErrorInService::DBError);
        let category_ids = models
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|m: &schema::CategoryModel| m.id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        // 根据分类查询订阅源，取前count个
        if let Some(count) = req.need_feed_logo_count {
            let category_ids_str = category_ids
                .iter()
                .map(|idf| format!("{}", idf))
                .collect::<Vec<_>>()
                .join(",");
            let subscription_raw_sql = format!(
                r#"
                SELECT id,  category_id, logo
                FROM (
                    SELECT
                    s.id AS id, s.title AS title, 
                    s.category_id AS category_id, s.logo AS logo,
                    ROW_NUMBER() OVER (PARTITION BY s.category_id ORDER BY s.sort_order DESC) AS sn
                    FROM rss_subscriptions s
                    WHERE s.category_id IN ({})
                ) AS subquery
                WHERE sn <= {};
                "#,
                category_ids_str, count
            );
            tracing::info!("category_ids_str:{}", category_ids_str);
            let subscription_stmt = Statement::from_sql_and_values(
                conn.get_database_backend(),
                subscription_raw_sql,
                vec![],
            );

            let ori_subscription_models = lib_entity::rss_subscriptions::Entity::find()
                .select_only()
                .column_as(lib_entity::rss_subscriptions::Column::Id, "id")
                .column_as(
                    lib_entity::rss_subscriptions::Column::Identifier,
                    "identifier",
                )
                .column_as(
                    lib_entity::rss_subscriptions::Column::CategoryId,
                    "category_id",
                )
                .column_as(lib_entity::rss_subscriptions::Column::Link, "link")
                .from_raw_sql(subscription_stmt)
                .into_json()
                .all(conn)
                .await
                .map_err(|e| {
                    tracing::error!("查询链接失败:{}", e);
                    e
                })?;

            for (i, m) in models.as_mut().unwrap().iter_mut().enumerate() {
                let category_id = m.id;
                // 只取前count个
                let urls = ori_subscription_models
                    .iter()
                    .filter_map(|feed| {
                        // get json category_id as i64
                        let feed_category_id = feed["category_id"].as_i64().unwrap_or_default();
                        if feed_category_id == category_id {
                            let link = feed["logo"].as_str().map(|s| s.to_string());
                            link
                        } else {
                            None
                        }
                    })
                    .take(count.try_into().unwrap())
                    .collect::<Vec<_>>();
                m.first_three_feed_urls = Some(urls);
            }
        }

        models
    }
}

#[cfg(test)]
mod tests {
    use migration::{Migrator, MigratorTrait};
    use sqlx::migrate::Migrate;

    use crate::rss::schema::QueryCategoryRequestBuilder;

    use super::*;
    #[tokio::test]
    async fn test_create_category() {
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let db = crate::get_db_conn(base_url).await;
        Migrator::up(&db, None).await.unwrap();

        let contoller = CategoryController;

        let req = CreateOrUpdateCategoryRequest {
            id: None,
            parent_id: None,
            title: "test".to_owned(),
            description: None,
            sort_order: Some(0),
        };
        let res = contoller.insert_category(req, &db).await.unwrap();
        assert_eq!(res.title, "test");

        let req = CreateOrUpdateCategoryRequest {
            id: Some(res.identifier),
            parent_id: None,
            title: "test_updated".to_owned(),
            description: None,
            sort_order: Some(0),
        };
        let res = contoller.insert_category(req, &db).await.unwrap();
        assert_eq!(res.title, "test_updated");
    }

    #[tokio::test]
    async fn test_query_category() {
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let conn = crate::get_db_conn(base_url).await;
        Migrator::up(&conn, None).await.unwrap();

        let contoller = CategoryController;
        let req = CreateOrUpdateCategoryRequest {
            id: None,
            parent_id: None,
            title: "test".to_owned(),
            description: None,
            sort_order: Some(0),
        };
        let res = contoller.insert_category(req, &conn).await.unwrap();
        assert_eq!(res.title, "test");

        let req = QueryCategoryRequestBuilder::default()
            .ids(vec![res.id])
            .build()
            .unwrap();
        let res = contoller.query_category(req, &conn).await.unwrap();
        assert_eq!(res.len(), 1);
    }
}
