use crate::{
    api_error::APIError, middlewares::auth_claim::AuthClaims, response::APIResponse, AppState,
};
use axum::{
    routing::{get, post},
    Router,
};
use axum::{Extension, Json};
use axum_extra::routing::RouterExt;
use lib_core::{
    common_schema::{PageRequest, PageResponse},
    error::ErrorInService,
    rss::{
        schema::{
            CategoryModel, CreateAiTokenRecordRequestBuilder, CreateOrUpdateCategoryRequest,
            CreateOrUpdateSubscriptionRequest, LinkMindMapModel, LinkMindMapRequest, LinkModel,
            LinkSummaryModel, LinkSummaryRequest, QueryCategoryRequest, QueryRssLinkRequest,
            QueryRssLinkRequestBuilder, QuerySubscriptionRequest,
            QuerySubscriptionsWithLinksRequest, SubscriptionModel, UpdateSubscriptionCountRequest,
        },
        CategoryController, LinkController, LinkSummaryController, SubscriptionController,
    },
};
use lib_openai::{AISummaryController, OpenAIConfig};
use lib_utils::Setting;
use serde_json::json;
use std::sync::Arc;

/// 创建/更新分类
///
/// 该方法用于创建或更新分类。
pub(crate) async fn update_category(
    app: Extension<Arc<AppState>>,
    Json(req): Json<CreateOrUpdateCategoryRequest>,
) -> Result<APIResponse<CategoryModel>, APIError> {
    let conn = &app.pool;
    let category_controller = CategoryController;
    let updated = category_controller.insert_category(req, conn).await?;
    Ok(APIResponse::<CategoryModel>::new()
        .with_code(200_i32)
        .with_data(updated))
}

/// 获取类别信息。
async fn query_categories_by_option(
    app: Extension<Arc<AppState>>,
    Json(req): Json<QueryCategoryRequest>,
) -> Result<APIResponse<Vec<CategoryModel>>, APIError> {
    let conn = &app.pool;
    let category_controller = CategoryController;
    let categories = category_controller.query_category(req, conn).await?;
    Ok(APIResponse::<Vec<CategoryModel>>::new()
        .with_code(200_i32)
        .with_data(categories))
}

// 查找订阅源
async fn query_rss_subscription_by_options(
    app: Extension<Arc<AppState>>,
    Json(find_rss_req): Json<QuerySubscriptionRequest>,
) -> Result<APIResponse<PageResponse<SubscriptionModel>>, APIError> {
    let pool = &app.pool;
    // 检测耗时
    let controller = SubscriptionController;
    let page_with_model = controller
        .query_subscription(find_rss_req, pool)
        .await
        .map_err(|e| {
            tracing::error!("query_rss_subscription_by_options error:{}", e);
            e
        })?;
    let page_info = page_with_model.map_data_into();
    Ok(APIResponse::<PageResponse<SubscriptionModel>>::new()
        .with_code(200_i32)
        .with_data(page_info))
}

// 创建/更新订阅源
async fn update_rss_subscription(
    app: Extension<Arc<AppState>>,
    Json(req): Json<CreateOrUpdateSubscriptionRequest>,
) -> Result<APIResponse<i64>, APIError> {
    let conn = &app.pool;

    let (_, updated) = SubscriptionController
        .insert_subscription(req, conn)
        .await?;
    Ok(APIResponse::<i64>::new()
        .with_code(200_i32)
        .with_data(updated))
}

// 查询订阅链接
async fn query_rss_links(
    app: Extension<Arc<AppState>>,
    Json(req): Json<QueryRssLinkRequest>,
) -> Result<APIResponse<PageResponse<LinkModel>>, APIError> {
    let conn = &app.pool;
    let page_with_model = LinkController.query_links(req, conn).await?;
    Ok(APIResponse::<PageResponse<LinkModel>>::new()
        .with_code(200_i32)
        .with_data(page_with_model))
}

/// 查询订阅链接数量
/// 提供 ids/ idfs/ title / 订阅源 / 发布时间范围 维度的查询
async fn query_rss_links_count(
    app: Extension<Arc<AppState>>,
    Json(req): Json<QueryRssLinkRequest>,
) -> Result<APIResponse<u64>, APIError> {
    let conn = &app.pool;
    let count = req.fetch_count(conn).await?;
    Ok(APIResponse::<u64>::new()
        .with_code(200_i32)
        .with_data(count))
}

/// 总结链接
async fn summary_rss_link(
    app: Extension<Arc<AppState>>,
    claims: AuthClaims,
    Json(req): Json<super::entities::SummaryLinkRequest>,
) -> Result<APIResponse<LinkSummaryModel>, APIError> {
    let conn = &app.pool;

    let mut summary_cache_query = LinkSummaryRequest {
        link_url: req.link_url.clone(),
        content: None,
    };

    let user = match claims.get_user(conn).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("summary_rss_link error:{}", e);
            return Err(APIError::ErrorParams("未找到用户".to_string()));
        }
    };

    let setting = Setting::global();

    let api_key = match setting.openai.api_key {
        Some(api_key) => api_key,
        None => {
            tracing::error!("summary_rss_link error:{}", "OpenAI API Key 未配置");
            return Err(ErrorInService::Custom("OpenAI API Key 未配置".to_string()).into());
        }
    };
    let mut openai_config = OpenAIConfig::default().with_api_key(api_key);
    if let Some(api_base) = setting.openai.api_base {
        openai_config = openai_config.with_api_base(api_base);
    }

    // 如果请求体里面已经包含了文章的正文内容了，则直接使用
    let content = match req.link_content {
        Some(content) => content,
        None => {
            let js_server_host = match setting.services.js_server_host {
                Some(js_server_host) => js_server_host,
                None => {
                    tracing::error!("summary_rss_link error:{}", "链接解析服务未就绪");
                    return Err(ErrorInService::Custom("链接解析服务未就绪".to_string()).into());
                }
            };
            let request_url = format!("{}/parse/md", js_server_host);
            let resp = reqwest::Client::new()
                .post(request_url)
                .json(&json!({ "url": req.link_url }))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("summary_rss_link error:{}", e);
                    ErrorInService::Custom("请求解析链接失败".to_string())
                })?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| {
                    tracing::error!("summary_rss_link error:{}", e);
                    ErrorInService::Custom("解析链接失败".to_string())
                })?;
            resp["content"]
                .as_str()
                .ok_or_else(|| {
                    tracing::error!("summary_rss_link error:{}", "链接解析服务返回内容提取失败");
                    ErrorInService::Custom("链接解析服务返回内容提取失败".to_string())
                })?
                .to_string()
        }
    };

    // 这里获得用户文章消耗的 token 数量， 并以此作为消费的凭证
    let controller = AISummaryController::<OpenAIConfig>::no_config();
    let content_token_cost = match controller
        .num_tokens_with_content(Some("gpt-3.5-turbo".to_string()), content.clone())
    {
        Ok(cost) => cost as i64,
        Err(e) => {
            // 否则使用 字数的 0.7
            let content_len = content.chars().count();
            // 返回
            let fake_token_cost = (content_len as f64 * 0.7) as i64;
            tracing::error!(
                "summary_rss_link cant get token cost:{}, use fake:{}",
                e,
                fake_token_cost
            );
            fake_token_cost
        }
    };

    let request = LinkSummaryRequest {
        link_url: req.link_url.clone(),
        content: Some(content.to_string()),
    };

    // see if the user has a summary cache
    let summary_controller = LinkSummaryController;

    let summary = match summary_controller
        .find_summary_by_link_url(summary_cache_query.link_url, conn)
        .await
    {
        Ok(Some(summary)) => {
            // 存在缓存的总结，直接返回
            summary.into()
        }
        _ => summary_controller
            .insert_link_summary(request, openai_config, conn)
            .await
            .map_err(|e| {
                tracing::error!("summary_rss_link on summary error:{}", e);
                e
            })?,
    };

    Ok(APIResponse::<LinkSummaryModel>::new()
        .with_code(200_i32)
        .with_data(summary))
}

async fn summary_article_mind_map(
    app: Extension<Arc<AppState>>,
    Json(req): Json<super::entities::SummaryLinkRequest>,
) -> Result<APIResponse<LinkMindMapModel>, APIError> {
    let conn = &app.pool;

    /// 查找是否存在缓存
    let summary_cache_query = LinkMindMapRequest {
        link_url: req.link_url.clone(),
        content: None,
    };
    let controller = LinkSummaryController;
    if let Ok(Some(summary)) = controller
        .find_mind_map_by_link_url(summary_cache_query.link_url.clone(), conn)
        .await
    {
        // 存在缓存的总结，直接返回
        return Ok(APIResponse::<LinkMindMapModel>::new()
            .with_code(200_i32)
            .with_data(summary.into()));
    }

    let setting = Setting::global();

    let api_key = match setting.openai.api_key {
        Some(api_key) => api_key,
        None => {
            tracing::error!("summary_rss_link error:{}", "OpenAI API Key 未配置");
            return Err(ErrorInService::Custom("OpenAI API Key 未配置".to_string()).into());
        }
    };
    let mut openai_config = OpenAIConfig::default().with_api_key(api_key);
    if let Some(api_base) = setting.openai.api_base {
        openai_config = openai_config.with_api_base(api_base);
    }
    // 如果请求体里面已经包含了文章的正文内容了，则直接使用
    let content = match req.link_content {
        Some(content) => content,
        None => {
            let js_server_host = match setting.services.js_server_host {
                Some(js_server_host) => js_server_host,
                None => {
                    tracing::error!("summary_rss_link error:{}", "链接解析服务未就绪");
                    return Err(ErrorInService::Custom("链接解析服务未就绪".to_string()).into());
                }
            };
            let request_url = format!("{}/parse", js_server_host);
            let resp = reqwest::Client::new()
                .post(request_url)
                .json(&json!({ "url": req.link_url }))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("summary_rss_link error:{}", e);
                    ErrorInService::Custom("请求解析链接失败".to_string())
                })?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| {
                    tracing::error!("summary_rss_link error:{}", e);
                    ErrorInService::Custom("解析链接失败".to_string())
                })?;
            resp["content"]
                .as_str()
                .ok_or_else(|| {
                    tracing::error!("summary_rss_link error:{}", "链接解析服务返回内容提取失败");
                    ErrorInService::Custom("链接解析服务返回内容提取失败".to_string())
                })?
                .to_string()
        }
    };

    let request = LinkMindMapRequest {
        link_url: req.link_url.clone(),
        content: Some(content.to_string()),
    };

    let summary = controller
        .insert_link_mindmap(openai_config, request, conn)
        .await?;
    Ok(APIResponse::<LinkMindMapModel>::new()
        .with_code(200_i32)
        .with_data(summary))
}

pub(crate) fn build_routes() -> axum::Router {
    Router::new()
        // 订阅源
        .route_with_tsr(
            "/subscrition/query",
            post(query_rss_subscription_by_options),
        )
        // 订阅源更新
        .route_with_tsr("/subscrition/update", post(update_rss_subscription))
        // 分类更新
        .route_with_tsr("/category/update", post(update_category))
        .route_with_tsr("/category/query", post(query_categories_by_option))
        // 链接查询
        .route_with_tsr("/link/query", post(query_rss_links))
        // 查询链接数量
        .route_with_tsr("/link/query_count", post(query_rss_links_count))
        // 总结链接
        .route_with_tsr("/link/summary", post(summary_rss_link))
        // 生成文章的思维导图
        .route_with_tsr("/link/mindmap", post(summary_article_mind_map))
}
