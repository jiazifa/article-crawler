use std::{collections::HashMap, ops::Sub};

use chrono::{NaiveDateTime, Timelike};
use lib_entity::{rss_subscription, rss_subscription_build_record, rss_subscription_config};
use lib_utils::math::{get_page_count, get_page_offset};

use super::{
    schema::{
        InsertSubscriptionRecordRequest, QuerySubscriptionConfigRequest,
        QuerySubscriptionRecordRequest, QuerySubscriptionRecordRequestBuilder,
        UpdateSubscriptionConfigRequest, UpdateSubscriptionConfigRequestBuilder,
    },
    CreateOrUpdateSubscriptionRequest, QueryPreferUpdateSubscriptionRequest,
    SubscriptionBuildSourceType,
};
use crate::{
    common_schema::{PageRequest, PageRequestBuilder, PageResponse},
    error::ErrorInService,
    DBConnection,
};
use sea_orm::{entity::*, query::*};

pub struct SubscritionConfigController;

impl SubscritionConfigController {
    pub async fn insert_subscription_config(
        &self,
        req: UpdateSubscriptionConfigRequest,
        conn: &DBConnection,
    ) -> Result<(), ErrorInService> {
        let origin_model = rss_subscription_config::Entity::find()
            .filter(rss_subscription_config::Column::SubscriptionId.eq(req.subscription_id))
            .one(conn)
            .await?;
        let need_update = origin_model.is_some();

        let mut new_model = match origin_model {
            Some(m) => m.into_active_model(),
            None => rss_subscription_config::ActiveModel {
                subscription_id: Set(req.subscription_id),
                initial_frequency: Set(req.initial_frequency.clone()),
                fitted_adaptive: Set(req.fitted_adaptive),
                ..Default::default()
            },
        };

        if req.fitted_adaptive {
            // 更新 fitted_frequency, 限制小数点后四位
            if let Some(fitted_frequency) = req.fitted_frequency {
                new_model.fitted_frequency = Set(Some(fitted_frequency));
            }
        } else {
            // 如果不是自适应, 则 fitted_frequency 为 initial_frequency
            new_model.fitted_frequency = Set(Some(req.initial_frequency));
        }

        // 更新 source type
        if let Some(source_type) = req.source_type {
            new_model.source_type = Set(source_type);
        }

        // 更新 last_build_at
        if let Some(last_build_at) = req.last_build_at {
            new_model.last_build_at = Set(Some(last_build_at));
        }

        _ = match need_update {
            true => new_model.update(conn).await?,
            false => new_model.insert(conn).await?,
        };
        Ok(())
    }
    // 更新订阅源的更新配置，基于订阅源的更新记录
    pub async fn update_subscription_config(
        &self,
        subscription_ids: Vec<i64>,
        conn: &DBConnection,
    ) -> Result<(), ErrorInService> {
        // 基于 records, 计算出时间间隔: 需要过滤出 status 为 0 的记录
        // 通过 subscription_id 分组, 计算出每个订阅源的更新时间间隔, 按照时间排序
        // 7天前的现在

        let now = chrono::Utc::now().naive_utc();
        let seven_days_ago = now - chrono::Duration::days(7);
        let req = QuerySubscriptionRecordRequestBuilder::default()
            .subscription_ids(subscription_ids)
            .status(vec![
                rss_subscription_build_record::Status::Success,
                rss_subscription_build_record::Status::MostSuccess,
            ])
            .create_time_lower(seven_days_ago)
            .create_time_upper(now)
            .page(PageRequest::max_page())
            .build()?;

        let records = self.query_subscription_record(req, conn).await?.data;

        let mut subscription_record_map: HashMap<i64, Vec<NaiveDateTime>> = HashMap::new();
        let mut last_build_at_map: HashMap<i64, NaiveDateTime> = HashMap::new();

        for record in records {
            let subscription_id = record.subscription_id;
            let create_time = record.created_at;
            let status = record.status;
            match status {
                rss_subscription_build_record::Status::Faild
                | rss_subscription_build_record::Status::Unknow
                | rss_subscription_build_record::Status::FweSuccess => {
                    subscription_record_map
                        .entry(subscription_id)
                        .or_insert(vec![])
                        .push(create_time);
                }
                _ => {
                    if let Some(records) = subscription_record_map.get_mut(&subscription_id) {
                        records.push(create_time);
                    }

                    // 比较最新的时间， 更新 last_build_at
                    if let Some(last_build_at) = last_build_at_map.get_mut(&subscription_id) {
                        if create_time > *last_build_at {
                            *last_build_at = create_time;
                        }
                    } else {
                        last_build_at_map.insert(subscription_id, create_time);
                    }
                }
            }
        }

        // 计算出每个订阅源的更新时间间隔
        let mut subscription_update_interval: HashMap<i64, f32> = HashMap::new();
        for (subscription_id, records) in subscription_record_map {
            // 先排序
            let mut records = records;
            records.sort();
            let mut interval = vec![];
            for i in 0..records.len() - 1 {
                // 计算出时间间隔，单位为分钟
                let time = records[i + 1].and_utc().timestamp() - records[i].and_utc().timestamp();
                // cast time to minutes
                let minute = time / 60;
                interval.push(minute);
            }
            // 去掉最大值和最小值
            if interval.len() > 2 {
                interval.sort();
                interval.pop();
                interval.remove(0);
            }
            if interval.is_empty() {
                continue;
            }
            let avg_interval = interval.iter().sum::<i64>() as f32 / interval.len() as f32;
            subscription_update_interval.insert(subscription_id, avg_interval);
        }
        // 更新订阅源的更新配置
        for (subscription_id, interval) in subscription_update_interval {
            let mut update_builder = UpdateSubscriptionConfigRequestBuilder::default();
            update_builder.subscription_id(subscription_id);
            update_builder.initial_frequency(3600.0);

            update_builder.fitted_frequency(interval);
            update_builder.fitted_adaptive(true);
            update_builder.source_type(SubscriptionBuildSourceType::Rss);

            if let Some(last_build_at) = last_build_at_map.get(&subscription_id) {
                update_builder.last_build_at(*last_build_at);
            }

            let update = update_builder.build()?;
            SubscritionConfigController
                .insert_subscription_config(update, conn)
                .await?;
        }
        Ok(())
    }

    pub async fn query_prefer_update_subscription(
        &self,
        req: QueryPreferUpdateSubscriptionRequest,
        conn: &DBConnection,
    ) -> Result<Vec<CreateOrUpdateSubscriptionRequest>, ErrorInService> {
        // 首先将 LastBuildDate 为空的全部查出来
        // 根据总数量, 计算出需要更新的数量
        // 然后根据时间排序, 时间越近的排在后面, 取出需要更新的数量

        // 实现查询一定比例数量的数据，根据 LastBuildDate 排序
        //例如: 有一百条数据,想查询 1/4, 其中有20条LastBuildDate为空，则我想获得 20 + (100-80) / 4 条数据. 应该如何实现?

        // 2. 查询出 LastBuildDate 不为空的数据
        // 3. 计算出需要查询的数量
        // 4. 根据 LastBuildDate 排序, 取出需要查询的数量
        // 5. 将两个结果合并

        // 1. 先查询出 LastBuildDate 为空的数据
        let all_count = rss_subscription::Entity::find().count(conn).await?;
        let null_last_build_date_count = rss_subscription::Entity::find()
            .left_join(rss_subscription_config::Entity)
            .left_join(rss_subscription_build_record::Entity)
            .filter(rss_subscription_config::Column::LastBuildAt.is_null())
            .count(conn)
            .await?;

        let limit_count = match req.expect_update_times {
            0 => 0,
            _ => {
                null_last_build_date_count
                    .checked_sub(
                        all_count
                            .checked_sub(null_last_build_date_count)
                            .unwrap_or(0),
                    )
                    .unwrap_or(0)
                    .checked_div(req.expect_update_times as u64)
                    .unwrap_or(0)
                    + null_last_build_date_count
            }
        };

        let mut select = rss_subscription::Entity::find()
            .left_join(rss_subscription_config::Entity)
            .order_by_asc(rss_subscription_config::Column::LastBuildAt)
            .limit(limit_count);

        select = select.select();
        let models = select.all(conn).await?;
        let mut reqs = models
            .iter()
            .map(|m| m.clone().into())
            .collect::<Vec<CreateOrUpdateSubscriptionRequest>>();

        tracing::info!(
            "期望{}次更新完，发现可能需要更新的订阅源数量: {}",
            req.expect_update_times,
            reqs.len()
        );
        // 过滤掉不需要更新的订阅源
        // 查询 订阅源配置
        let now = chrono::Utc::now().naive_utc();
        let config_req = QuerySubscriptionConfigRequest::new(reqs.iter().map(|r| r.id).collect());
        if let Ok(configs) = self.query_subscription_config(config_req, conn).await {
            let config_map = configs
                .iter()
                .map(|c| (c.subscription_id, c.clone()))
                .collect::<HashMap<i64, rss_subscription_config::Model>>();

            reqs.retain(|r| {
                if let Some(id) = r.id {
                    if let Some(config) = config_map.get(&id) {
                        let fitted_frequency = config.get_frequency();
                        let mut max_frequency = config.initial_frequency;
                        if let Some(fitted_frequency) = config.fitted_frequency {
                            max_frequency = fitted_frequency * 1.1;
                        }
                        if let Some(last_update_time) = r.last_build_date {
                            // 计算出时间间隔, 如果时间间隔小于频率, 则不需要更新
                            // 即: 如果时间间隔接近或者大于频率, 则需要更新
                            let interval =
                                now.and_utc().timestamp() - last_update_time.and_utc().timestamp();
                            let minute = interval / 60;
                            if minute < (fitted_frequency * 0.9) as i64 {
                                return false;
                            }
                        }
                    }
                }
                true
            });
        }
        println!("最终需要更新的订阅源数量: {}", reqs.len());
        Ok(reqs)
    }

    // 添加订阅源的更新记录
    pub async fn insert_subscription_update_record(
        &self,
        req: InsertSubscriptionRecordRequest,
        conn: &DBConnection,
    ) -> Result<rss_subscription_build_record::Model, ErrorInService> {
        let mut model = rss_subscription_build_record::ActiveModel {
            subscription_id: Set(req.subscription_id),
            identifier: Set(uuid::Uuid::new_v4().simple().to_string()),
            ..Default::default()
        };

        model.status = Set(req.status);

        model.remark = Set("".to_string());
        if let Some(create_at) = req.create_time {
            model.created_at = Set(create_at);
        }

        let inserted = model.insert(conn).await?;
        Ok(inserted)
    }

    // 查询订阅源的更新记录
    pub async fn query_subscription_record(
        &self,
        req: QuerySubscriptionRecordRequest,
        conn: &DBConnection,
    ) -> Result<PageResponse<rss_subscription_build_record::Model>, ErrorInService> {
        let mut select = rss_subscription_build_record::Entity::find();
        if let Some(subscription_ids) = &req.subscription_ids {
            if !subscription_ids.is_empty() {
                select = select.filter(
                    rss_subscription_build_record::Column::SubscriptionId
                        .is_in(subscription_ids.clone()),
                );
            }
        }
        select = select.order_by_desc(rss_subscription_build_record::Column::CreatedAt);

        // filter with time
        if let Some(start_time) = &req.create_time_lower {
            select =
                select.filter(rss_subscription_build_record::Column::CreatedAt.gt(*start_time));
        }
        if let Some(end_time) = &req.create_time_upper {
            select = select.filter(rss_subscription_build_record::Column::CreatedAt.lt(*end_time));
        }

        let page_info = req.page.clone();
        let page_size = page_info.page_size;
        let page = page_info.page;
        let offset = get_page_offset(page, page_size);

        select = select.limit(page_size).offset(offset);
        let all_count = select.clone().count(conn).await.unwrap_or(0);
        let page_count = get_page_count(all_count, page_size);
        let models = select.all(conn).await?;
        let page_response = PageResponse::new(page_count, page, page_size, models);
        Ok(page_response)
    }

    pub async fn query_subscription_config(
        &self,
        req: QuerySubscriptionConfigRequest,
        conn: &DBConnection,
    ) -> Result<Vec<rss_subscription_config::Model>, ErrorInService> {
        let mut select = rss_subscription_config::Entity::find();
        if let Some(subscription_ids) = &req.subscription_ids {
            if !subscription_ids.is_empty() {
                select = select.filter(
                    rss_subscription_config::Column::SubscriptionId.is_in(subscription_ids.clone()),
                );
            }
        }
        let model = select.all(conn).await?;
        Ok(model)
    }
}

// tests
#[cfg(test)]
mod tests {
    use crate::{
        common_schema::PageRequest,
        rss::{
            category_service::CategoryController, schema::InsertSubscriptionRecordRequestBuilder,
            subscription_service::SubscriptionController,
        },
    };

    use super::*;
    use chrono::Days;
    use migration::{Migrator, MigratorTrait};
    use sqlx::migrate::Migrate;

    #[tokio::test]
    async fn test_insert_subscription_record() {
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:?mode=rwc".to_owned());
        let db = crate::get_db_conn(base_url).await;
        Migrator::up(&db, None).await.unwrap();

        // 1 周前到现在的时间，间隔一天

        let dates = (0..7)
            .map(|i| {
                chrono::Utc::now()
                    .checked_sub_days(Days::new(i))
                    .unwrap()
                    .naive_utc()
            })
            .collect::<Vec<_>>();

        let controller = SubscritionConfigController;
        // insert records
        for date in dates.clone() {
            let req = InsertSubscriptionRecordRequest {
                subscription_id: 1,
                status: rss_subscription_build_record::Status::Success,
                create_time: Some(date),
            };
            let record = controller
                .insert_subscription_update_record(req, &db)
                .await
                .unwrap();
            assert_eq!(record.subscription_id, 1);
        }

        // query records
        let query = QuerySubscriptionRecordRequest {
            subscription_ids: Some(vec![1]),
            status: None,
            create_time_lower: Some(dates[6]),
            create_time_upper: Some(dates[0]),
            page: PageRequest::max_page(),
        };
        let records = controller
            .query_subscription_record(query, &db)
            .await
            .unwrap();
        assert_eq!(records.data.len(), 5);
    }

    #[tokio::test]
    async fn test_insert_subscription_config() {
        let db = crate::test_runner::setup_database().await;

        let req = UpdateSubscriptionConfigRequestBuilder::default()
            .subscription_id(1)
            .initial_frequency(1.0)
            .fitted_frequency(1.0)
            .fitted_adaptive(true)
            .build()
            .unwrap();
        SubscritionConfigController
            .insert_subscription_config(req, &db)
            .await
            .unwrap();

        let config = rss_subscription_config::Entity::find()
            .filter(rss_subscription_config::Column::SubscriptionId.eq(1))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(config.subscription_id, 1);
    }

    #[tokio::test]
    async fn test_insert_subscription_config2() {
        let conn = crate::test_runner::setup_database().await;

        // time for now
        let now = chrono::Utc::now().naive_utc();
        // 1 hour age
        let one_hour_ago = now - chrono::Duration::hours(1);
        let category_controller = CategoryController;
        let insert_category_req =
            crate::rss::schema::CreateOrUpdateCategoryRequestBuilder::default()
                .title("test")
                .build()
                .unwrap();
        let category = category_controller
            .insert_category(insert_category_req, &conn)
            .await
            .unwrap();
        let subs_update_controller = SubscritionConfigController;
        let subs_controller = SubscriptionController;
        let insert_subscription_req =
            crate::rss::schema::CreateOrUpdateSubscriptionRequestBuilder::default()
                .category_id(category.id)
                .link("http://www.baidu.com")
                .title("baidu")
                .link("http://www.baidu.com")
                .last_build_date(one_hour_ago)
                .build()
                .unwrap();
        let sub_model = subs_controller
            .insert_subscription(insert_subscription_req, &conn)
            .await
            .unwrap();
        let query_subscription_req = crate::rss::schema::QuerySubscriptionRequestBuilder::default()
            .ids(vec![sub_model.1])
            .build()
            .unwrap();
        let query_subscription_res = subs_controller
            .query_subscription(query_subscription_req, &conn)
            .await
            .unwrap();
        let subscription = query_subscription_res.data.first().unwrap();

        // insert update record
        let insert_subscription_record_req = InsertSubscriptionRecordRequestBuilder::default()
            .subscription_id(subscription.id)
            .status(rss_subscription_build_record::Status::Success)
            .create_time(one_hour_ago)
            .build()
            .unwrap();

        _ = subs_update_controller
            .insert_subscription_update_record(insert_subscription_record_req, &conn)
            .await
            .unwrap();

        // insert update record for now
        let insert_subscription_record_req = InsertSubscriptionRecordRequestBuilder::default()
            .subscription_id(subscription.id)
            .status(rss_subscription_build_record::Status::Success)
            .create_time(now)
            .build()
            .unwrap();
        _ = subs_update_controller
            .insert_subscription_update_record(insert_subscription_record_req, &conn)
            .await
            .unwrap();

        let insert_subscription_config_req = UpdateSubscriptionConfigRequestBuilder::default()
            .subscription_id(subscription.id)
            .initial_frequency(1.0)
            .fitted_frequency(60.0)
            .fitted_adaptive(true)
            .build()
            .unwrap();

        subs_update_controller
            .insert_subscription_config(insert_subscription_config_req, &conn)
            .await
            .unwrap();

        let query_preference_update_subscription_req = QueryPreferUpdateSubscriptionRequest::new(1);
        let query_preference_update_subscription_res = subs_update_controller
            .query_prefer_update_subscription(query_preference_update_subscription_req, &conn)
            .await
            .unwrap();
        assert!(!query_preference_update_subscription_res.is_empty());

        // 当修改这个配置，时间间隔变长后，应该查询不到
        let insert_subscription_config_req = UpdateSubscriptionConfigRequestBuilder::default()
            .subscription_id(subscription.id)
            .initial_frequency(1.0)
            .fitted_frequency(100.0)
            .fitted_adaptive(true)
            .build()
            .unwrap();

        subs_update_controller
            .insert_subscription_config(insert_subscription_config_req, &conn)
            .await
            .unwrap();

        let query_preference_update_subscription_req = QueryPreferUpdateSubscriptionRequest::new(1);
        let query_preference_update_subscription_res = subs_update_controller
            .query_prefer_update_subscription(query_preference_update_subscription_req, &conn)
            .await
            .unwrap();
        assert_eq!(query_preference_update_subscription_res.len(), 1);
    }
}
