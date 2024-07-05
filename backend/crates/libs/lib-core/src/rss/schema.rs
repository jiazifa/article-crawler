use crate::common_schema::PageRequest;
use crate::error::ErrorInService;

use chrono::naive::serde::ts_milliseconds_option;
use chrono::NaiveDateTime;
use lib_crawler::{try_get_all_image_from_html_content, try_get_all_text_from_html_content};
use lib_entity::rss_category;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

// 用于传递给外部的整理过的订阅源数据
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct SubscriptionModel {
    // id
    pub id: i64,
    // 唯一标识
    pub identifier: String,
    // 标题
    pub title: String,
    // Rss 链接地址
    pub link: String,
    // 分类Id
    pub category_id: i64,
    // 描述（可能包含 html）
    pub description: Option<String>,
    // 对应rss链接提供方的网站
    pub site_link: Option<String>,
    // icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    // visualUrl
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_url: Option<String>,
    // 语言
    pub language: Option<String>,
    // 评分
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    // 最后更新时间
    pub last_build_date: Option<NaiveDateTime>,
    // 主题色
    pub accent_color: Option<String>,
    // 本周文章数量
    pub article_count_for_this_week: Option<i32>,
    // 是否已经完成
    pub is_completed: Option<bool>,
    // 订阅者数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers: Option<i32>,
    // 用户数据
    // 订阅时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_at: Option<NaiveDateTime>,
    // 自定义标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    // 排序序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<LinkModel>>,
}

impl FromQueryResult for SubscriptionModel {
    fn from_query_result(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Self, sea_orm::prelude::DbErr> {
        let link_models: Option<Vec<LinkModel>> =
            serde_json::from_str(&res.try_get::<String>(pre, "links").unwrap_or("".into()))
                .unwrap_or(None);
        let mut model_builder = SubscriptionModelBuilder::default();
        let model = model_builder
            .id(res.try_get(pre, "id")?)
            .identifier(res.try_get(pre, "identifier")?)
            .title(res.try_get(pre, "title")?)
            .link(res.try_get(pre, "link")?)
            .category_id(res.try_get(pre, "category_id")?)
            .description(res.try_get(pre, "description").unwrap_or(None))
            .site_link(res.try_get(pre, "site_link").unwrap_or(None))
            .icon(res.try_get(pre, "icon").unwrap_or(None))
            .logo(res.try_get(pre, "logo").unwrap_or(None))
            .visual_url(res.try_get(pre, "visual_url").unwrap_or(None))
            .language(res.try_get(pre, "language").unwrap_or(None))
            .rating(res.try_get(pre, "rating").unwrap_or(None))
            .last_build_date(res.try_get(pre, "last_build_date").unwrap_or(None))
            .accent_color(res.try_get(pre, "accent_color").unwrap_or(None))
            .article_count_for_this_week(
                res.try_get(pre, "article_count_for_this_week")
                    .unwrap_or(None),
            )
            .is_completed(res.try_get(pre, "is_completed").unwrap_or(None))
            .subscribers(res.try_get(pre, "subscribers").unwrap_or(None))
            .subscribed_at(res.try_get(pre, "subscribed_at").unwrap_or(None))
            .custom_title(res.try_get(pre, "custom_title").unwrap_or(None))
            .sort_order(res.try_get(pre, "sort_order").unwrap_or(None))
            .links(link_models)
            .build()
            .map_err(|e| {
                sea_orm::prelude::DbErr::Custom(format!("build SubscriptionModel error:{}", e))
            })?;
        Ok(model)
    }

    fn from_query_result_optional(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Option<Self>, sea_orm::prelude::DbErr> {
        if let Ok(model) = SubscriptionModel::from_query_result(res, pre) {
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

// 作者
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// 用于传递给外部的整理过的链接数据
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct LinkModel {
    // id
    pub id: i64,
    // 唯一标识
    pub identifier: String,
    // 标题
    pub title: String,
    // 订阅源
    pub subscrption_id: i64,
    // 链接
    pub link: String,
    // 内容(可能为空，包含 html)
    #[builder(default)]
    pub content: Option<String>,
    // 描述 不包含 html
    #[builder(default)]
    pub description: Option<String>,
    // 发布时间
    #[builder(default)]
    pub published_at: Option<NaiveDateTime>,
    // 使用 serde 的 skip 反序列化，但自定义序列化
    #[serde(deserialize_with = "deserialize_authors_json")]
    #[builder(default)]
    pub authors: Option<Vec<Author>>,
    #[serde(deserialize_with = "deserialize_images_json")]
    #[builder(default)]
    pub images: Option<Vec<Image>>,
}

impl From<lib_entity::rss_links::Model> for LinkModel {
    fn from(value: lib_entity::rss_links::Model) -> Self {
        let authors: Option<Vec<Author>> =
            serde_json::from_str(&value.authors_json.unwrap_or("".to_string())).unwrap_or(None);
        let images: Option<Vec<Image>> =
            serde_json::from_str(&value.images.unwrap_or("".to_string())).unwrap_or(None);
        let description = value.description.unwrap_or("".to_string());
        let text_desc = value.desc_pure_txt.unwrap_or("".to_string());
        Self {
            id: value.id,
            identifier: value.identifier,
            title: value.title,
            subscrption_id: value.subscrption_id,
            link: value.link,
            content: Some(description),
            description: Some(text_desc),
            published_at: value.published_at,
            authors,
            images,
        }
    }
}

impl FromQueryResult for LinkModel {
    fn from_query_result(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Self, sea_orm::prelude::DbErr> {
        let authors: Option<Vec<Author>> =
            serde_json::from_str(&res.try_get::<String>(pre, "authors")?).unwrap_or(None);
        let images: Option<Vec<Image>> =
            serde_json::from_str(&res.try_get::<String>(pre, "images")?).unwrap_or(None);
        let description = res
            .try_get::<String>(pre, "description")
            .unwrap_or("".to_string());

        let text_desc = res
            .try_get::<String>(pre, "desc_pure_txt")
            .unwrap_or("".to_string());

        let mut model_builder = LinkModelBuilder::default();
        let model = model_builder
            .id(res.try_get(pre, "id")?)
            .identifier(res.try_get(pre, "identifier")?)
            .title(res.try_get(pre, "title")?)
            .subscrption_id(res.try_get(pre, "subscrption_id")?)
            .link(res.try_get(pre, "link")?)
            .content(Some(description))
            .description(Some(text_desc))
            .published_at(res.try_get(pre, "published_at").unwrap_or(None))
            .authors(authors)
            .images(images)
            .build()
            .map_err(|e| sea_orm::prelude::DbErr::Custom(format!("build LinkModel error:{}", e)))?;

        Ok(model)
    }

    fn from_query_result_optional(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Option<Self>, sea_orm::prelude::DbErr> {
        if let Ok(model) = LinkModel::from_query_result(res, pre) {
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

fn deserialize_authors_json<'de, D>(deserializer: D) -> Result<Option<Vec<Author>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let authors: Vec<Author> =
                serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
            Ok(Some(authors))
        }
        None => Ok(None),
    }
}

fn deserialize_images_json<'de, D>(deserializer: D) -> Result<Option<Vec<Image>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let images: Vec<Image> = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
            Ok(Some(images))
        }
        None => Ok(Some(Vec::new())),
    }
}

// 连接文章的总结
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkSummaryModel {
    pub link_url: String,
    // 提供者 例如 (gpt-3.5-turbo)
    pub provider: Option<String>,
    // 一句话总结
    pub one_sentence_summary: String,
    // 关键点
    pub key_points: Vec<String>,
    // 行动项
    pub action_items: Vec<String>,
    // 关键词
    pub keywords: Vec<String>,
}

impl From<lib_entity::rss_link_summary::Model> for LinkSummaryModel {
    fn from(value: lib_entity::rss_link_summary::Model) -> Self {
        // parse str to array
        let key_points = match serde_json::from_str::<Vec<String>>(
            &value.key_points.unwrap_or("[]".to_string()),
        ) {
            Ok(key_points) => key_points,
            Err(_) => Vec::new(),
        };
        let action_items = match serde_json::from_str::<Vec<String>>(
            &value.action_items.unwrap_or("[]".to_string()),
        ) {
            Ok(action_items) => action_items,
            Err(_) => Vec::new(),
        };
        let keywords = match serde_json::from_str::<Vec<String>>(
            &value.keywords.unwrap_or("[]".to_string()),
        ) {
            Ok(keywords) => keywords,
            Err(_) => Vec::new(),
        };
        let summary = match value.summary {
            Some(summary) => summary,
            None => "".to_string(),
        };

        Self {
            link_url: value.link_url,
            provider: value.provider,
            one_sentence_summary: summary,
            key_points,
            action_items,
            keywords,
        }
    }
}

/// 链接文章的思维导图
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkMindMapModel {
    pub link_url: String,
    // 总结版本
    pub version: String,
    // 总结语言
    pub language: String,
    // 思维导图的 markdown
    pub mind_map: String,
}

impl From<lib_entity::rss_link_mindmap::Model> for LinkMindMapModel {
    fn from(value: lib_entity::rss_link_mindmap::Model) -> Self {
        Self {
            link_url: value.link_url,
            version: value.version,
            language: value.language,
            mind_map: value.mind_map,
        }
    }
}

// 分类
#[derive(Debug, Clone, Serialize, Builder)]
pub struct CategoryModel {
    // id
    pub id: i64,
    // 唯一标识
    pub identifier: String,
    // 标题
    pub title: String,
    // 描述
    #[builder(default)]
    pub description: Option<String>,
    // 父分类
    #[builder(default)]
    pub parent_id: Option<i64>,
    // 排序序列
    #[builder(default)]
    pub sort_order: Option<i64>,

    // 前三个订阅源的链接地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_three_feed_urls: Option<Vec<String>>,
}

impl FromQueryResult for CategoryModel {
    fn from_query_result(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Self, sea_orm::prelude::DbErr> {
        let m_id = res.try_get::<i64>(pre, "id")?;
        let m_urls_str = res
            .try_get::<String>(pre, "first_three_feed_urls")
            .unwrap_or("".into());
        let m_urls: Option<Vec<String>> = serde_json::from_str(&m_urls_str).unwrap_or(None);

        let mut model_builder = CategoryModelBuilder::default();

        let model = model_builder
            .id(m_id)
            .identifier(res.try_get(pre, "identifier")?)
            .title(res.try_get(pre, "title")?)
            .description(res.try_get(pre, "description").unwrap_or(None))
            .parent_id(res.try_get(pre, "parent_id").unwrap_or(None))
            .sort_order(res.try_get(pre, "sort_order").unwrap_or(None))
            .first_three_feed_urls(m_urls)
            .build()
            .map_err(|e| {
                sea_orm::prelude::DbErr::Custom(format!("build CategoryModel error:{}", e))
            })?;
        Ok(model)
    }

    fn from_query_result_optional(
        res: &sea_orm::prelude::QueryResult,
        pre: &str,
    ) -> Result<Option<Self>, sea_orm::prelude::DbErr> {
        if let Ok(model) = CategoryModel::from_query_result(res, pre) {
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

impl From<rss_category::Model> for CategoryModel {
    fn from(value: rss_category::Model) -> Self {
        Self {
            id: value.id,
            identifier: value.identifier,
            title: value.title,
            description: value.description,
            parent_id: value.parent_id,
            sort_order: value.sort_order,
            first_three_feed_urls: None,
        }
    }
}

// 构建分类的请求
#[derive(Debug, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct CreateOrUpdateCategoryRequest {
    pub id: Option<i64>,
    // 标题
    pub title: String,
    // 描述
    pub description: Option<String>,
    // 父分类
    pub parent_id: Option<i64>,
    // 排序序列
    // 驼峰命名法
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: Option<i64>,
}

// 查找分类的请求
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct QueryCategoryRequest {
    // 唯一标识
    pub ids: Option<Vec<i64>>,
    // 标题
    pub title: Option<String>,
    // 描述
    pub description: Option<String>,
    // 父分类
    pub parent_ids: Option<Vec<i64>>,
    // 是否需要订阅源的链接，需要几个
    pub need_feed_logo_count: Option<u64>,
    // 分页
    pub page: Option<PageRequest>,
}

// 订阅源的更新请求
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct CreateOrUpdateSubscriptionRequest {
    pub id: Option<i64>,
    // 唯一标识
    pub identifier: Option<String>,
    // 标题
    pub title: String,
    // 描述
    pub description: Option<String>,
    // Rss 链接地址
    pub link: String,
    // 对应rss链接提供方的网站
    pub site_link: Option<String>,
    // 分类Id
    pub category_id: Option<i64>,
    // logo
    pub logo: Option<String>,
    // visual_url
    pub visual_url: Option<String>,
    // 语言
    pub language: Option<String>,
    // 评分
    pub rating: Option<i32>,
    // 发布日期
    #[serde(default)]
    #[serde(with = "ts_milliseconds_option")]
    pub pub_date: Option<NaiveDateTime>,
    // 最近更新时间
    #[serde(default)]
    #[serde(with = "ts_milliseconds_option")]
    pub last_build_date: Option<NaiveDateTime>,
    // 排序序列
    pub sort_order: Option<i32>,
}

impl From<lib_entity::rss_subscriptions::Model> for CreateOrUpdateSubscriptionRequest {
    fn from(value: lib_entity::rss_subscriptions::Model) -> Self {
        let mut req = CreateOrUpdateSubscriptionRequestBuilder::default();

        req.identifier(value.identifier);
        req.id(value.id);
        req.title(value.title);
        if let Some(value) = value.description {
            req.description(value);
        }
        if let Some(value) = value.link {
            req.link(value);
        }
        if let Some(value) = value.site_link {
            req.site_link(value);
        }
        if let Some(value) = value.category_id {
            req.category_id(value);
        }
        if let Some(value) = value.logo {
            req.logo(value);
        }
        if let Some(value) = value.visual_url {
            req.visual_url(value);
        }
        if let Some(value) = value.language {
            req.language(value);
        }
        if let Some(value) = value.rating {
            req.rating(value);
        }
        if let Some(value) = value.pub_date {
            req.pub_date(value);
        }
        if let Some(value) = value.last_build_date {
            req.last_build_date(value);
        }
        req.build()
            .expect(" `rss_subscriptions::Model` 解析 `CreateOrUpdateSubscriptionRequest` 失败")
    }
}

// 构建查找订阅源的请求
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct QuerySubscriptionRequest {
    // 唯一标识
    pub ids: Option<Vec<i64>>,
    // 唯一标识
    pub idfs: Option<Vec<String>>,
    // 标题
    pub title: Option<String>,
    // 描述
    pub description: Option<String>,
    // 分类Id
    pub category_id: Option<String>,
    // 语言
    pub language: Option<Vec<String>>,

    pub page: Option<PageRequest>,
}

// 订阅源的请求 + 链接的请求 = 订阅源的响应

#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct SubscriptionWithLinksResp {
    pub subscription: CreateOrUpdateSubscriptionRequest,
    pub links: Vec<CreateOrUpdateRssLinkRequest>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct QueryPreferUpdateSubscriptionRequest {
    // 期望几次更新完毕
    pub expect_update_times: u32,
}

impl QueryPreferUpdateSubscriptionRequest {
    pub fn new(expect_update_times: u32) -> Self {
        Self {
            expect_update_times,
        }
    }
}

// 订阅源更新状态枚举
#[derive(Debug, Clone, Deserialize, Default)]
pub enum SubscriptionUpdateStatus {
    #[default]
    Success,
    Failed(String),
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
// 添加订阅源的更新记录
pub struct InsertSubscriptionRecordRequest {
    // 订阅源Id
    pub subscription_id: i64,
    // 状态 表示订阅源此次更新的状态， 成功 / 失败 / 其他 如果是失败，需要记录失败原因
    pub status: SubscriptionUpdateStatus,
    // 创建时间
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(build_fn(error = "ErrorInService"))]
// 查找订阅源的更新记录
pub struct QuerySubscriptionRecordRequest {
    pub subscription_ids: Option<Vec<i64>>,
    // 状态列表
    pub status: Option<Vec<SubscriptionUpdateStatus>>,
    // 时间范围 低值  毫秒 13 位
    #[serde(default, with = "ts_milliseconds_option")]
    pub create_time_lower: Option<NaiveDateTime>,
    // 时间范围 高值  毫秒 13 位
    #[serde(default, with = "ts_milliseconds_option")]
    pub create_time_upper: Option<NaiveDateTime>,
    // 分页
    pub page: PageRequest,
}

#[derive(Debug, Clone, Deserialize)]
// 更新订阅源的配置
// 频率的定义: 一般来说，频率是一个浮点数，表示多少分钟更新一次 例如 60.0 表示一小时更新一次 30.0 表示半小时更新一次
pub struct UpdateSubscriptionConfigRequest {
    pub subscription_id: i64,
    pub initial_frequency: f32,
    pub fitted_frequency: Option<f32>,
    pub adaptive: bool,
}

impl UpdateSubscriptionConfigRequest {
    pub fn new(
        subscription_id: i64,
        init_frequency: Option<f32>,
        fitted_frequency: Option<f32>,
        adaptive: bool,
    ) -> Self {
        // 限制小数点后两位
        let init_frequency = match init_frequency {
            Some(f) => (f * 100.0).round() / 100.0,
            None => 1.0,
        };
        let fitted_frequency = fitted_frequency.map(|f| (f * 100.0).round() / 100.0);
        Self {
            subscription_id,
            initial_frequency: init_frequency,
            fitted_frequency,
            adaptive,
        }
    }
}

pub struct QuerySubscriptionConfigRequest {
    pub subscription_ids: Option<Vec<i64>>,
}

impl QuerySubscriptionConfigRequest {
    pub fn new(subscription_ids: Option<Vec<i64>>) -> Self {
        Self { subscription_ids }
    }
}

// - MARK:- 查找分类的请求, 会附带几条最新的链接
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct QuerySubscriptionsWithLinksRequest {
    pub category_id: i64,

    pub link_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct UpdateSubscriptionCountRequest {
    pub subscription_id: i64,
    pub offset: i64,
}

// 链接的更新请求
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct CreateOrUpdateRssLinkRequest {
    // 唯一标识
    pub identifier: Option<String>,
    // 标题
    pub title: String,
    // 订阅源
    pub subscrption_id: Option<i64>,
    // 链接
    pub link: String,
    // 描述
    pub description: Option<String>,
    // 纯文本描述
    pub desc_pure_txt: Option<String>,
    // 发布时间
    pub published_at: Option<NaiveDateTime>,
    // 作者
    pub authors_json: Option<String>,
    // 图片
    pub images_json: Option<String>,
}

// 构建查找链接的请求
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "ErrorInService"))]
pub struct QueryRssLinkRequest {
    // 唯一标识
    pub ids: Option<Vec<i64>>,
    // 唯一标识
    pub idfs: Option<Vec<String>>,
    // 标题
    pub title: Option<String>,
    // 订阅源 ids
    pub subscrption_ids: Option<Vec<i64>>,

    // 时间范围 低值  毫秒 13 位
    #[builder(default = "Option::None")]
    #[serde(default)]
    #[serde(with = "ts_milliseconds_option")]
    pub published_at_lower: Option<NaiveDateTime>,
    // 时间范围 高值  毫秒 13 位
    #[builder(default = "Option::None")]
    #[serde(default)]
    #[serde(with = "ts_milliseconds_option")]
    pub published_at_upper: Option<NaiveDateTime>,
    // 分页信息
    pub page: Option<PageRequest>,
}

// 构建总结文章的请求
#[derive(Debug, Deserialize)]
pub struct LinkSummaryRequest {
    pub link_url: String,
    pub content: Option<String>,
}

// 构建文章思维导图的请求
#[derive(Debug, Deserialize)]
pub struct LinkMindMapRequest {
    pub link_url: String,
    pub content: Option<String>,
}

// 创建AI 总结 Token 消耗记录
#[derive(Debug, Clone, Deserialize, Default, Builder)]
#[builder(setter(into, strip_option), default)]
#[builder(build_fn(error = "ErrorInService"))]
pub struct CreateAiTokenRecordRequest {
    pub customer_id: i64,
    pub amount: i32,
    pub transaction_order_number: Option<String>,
    pub provider: String,
    pub article_url: String,
    pub remark: Option<String>,
}

#[cfg(test)]
mod tests {

    use chrono::naive::serde::ts_milliseconds_option;
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize}; // Add this line to import the DateTime type

    #[derive(Serialize, Deserialize)]
    struct S {
        #[serde(default, with = "ts_milliseconds_option")]
        time: Option<NaiveDateTime>,
    }

    #[test]
    fn test_serde() {
        let s = S {
            time: Some(chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#"{"time":0}"#);

        let s: S = serde_json::from_str(r#"{"time":0}"#).unwrap();
        assert_eq!(
            s.time,
            Some(chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc())
        );

        let s: S = serde_json::from_str(r#"{"time":null}"#).unwrap();
        assert_eq!(s.time, None);

        let s: S = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(s.time, None);
    }
}
