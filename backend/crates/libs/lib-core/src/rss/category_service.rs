use std::collections::HashMap;

use crate::common_schema::{PageRequest, PageRequestBuilder, PageResponse};

use super::schema::{self, CreateOrUpdateCategoryRequest, QueryCategoryRequest};
use crate::error::ErrorInService;

use crate::DBConnection;
use lib_entity::feed_category;
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
            Some(idf) => feed_category::Entity::find().filter(feed_category::Column::Id.eq(idf)),
            None => feed_category::Entity::find()
                .filter(feed_category::Column::Title.eq(req.title.clone())),
        };

        let category = query.one(conn).await.map_err(ErrorInService::DBError)?;
        let prefer_update = category.is_some();
        let mut new_model = match category {
            Some(m) => m.into_active_model(),
            None => feed_category::ActiveModel {
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
        let mut select = feed_category::Entity::find();
        if let Some(ids) = &req.ids {
            if !ids.is_empty() {
                select = select.filter(feed_category::Column::Id.is_in(ids.clone()))
            }
        }
        if let Some(title) = &req.title {
            select = select.filter(feed_category::Column::Title.like(format!("%{}%", title)))
        }
        if let Some(description) = &req.description {
            select =
                select.filter(feed_category::Column::Description.like(format!("%{}%", description)))
        }
        if let Some(parent_ids) = &req.parent_ids {
            if !parent_ids.is_empty() {
                select = select.filter(feed_category::Column::ParentId.is_in(parent_ids.clone()))
            }
        }

        // 根据Sort order 和时间 排序
        select = select.order_by_desc(feed_category::Column::SortOrder);
        select = select.order_by_desc(feed_category::Column::UpdatedAt);

        let mut models = select
            .into_model()
            .all(conn)
            .await
            .map_err(ErrorInService::DBError);

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
        let db = crate::test_runner::setup_database().await;

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
            id: Some(res.id),
            parent_id: None,
            title: "test_updated".to_owned(),
            description: None,
            sort_order: Some(0),
        };
        let updated = contoller.insert_category(req, &db).await.unwrap();
        assert_eq!(updated.title, "test_updated");
        assert_eq!(res.id, updated.id);
    }

    #[tokio::test]
    async fn test_query_category() {
        let conn = crate::test_runner::setup_database().await;

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
