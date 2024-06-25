use axum_extra::routing::RouterExt;
use lib_core::{get_db_conn, DBConnection};
// use middlewares::verification::VerificationHeaderFields;
// use middlewares::VerificationHeaderFields;
use response::APIResponse;
use std::{sync::Arc, time::Duration};
use tower_http::add_extension::AddExtensionLayer;

use axum::{
    extract::{MatchedPath, Request},
    routing::get,
    Extension, Router,
};
use tower_http::{
    compression::{predicate::NotForContentType, CompressionLayer, DefaultPredicate, Predicate},
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
mod api_error;
mod middlewares;
mod response;
mod route;
mod rss;
mod utils;
use lib_utils::Setting;

#[derive(Debug)]
pub struct AppState {
    pub pool: DBConnection,
    pub setting: Setting,
}

async fn handler_404() -> Result<APIResponse<()>, api_error::APIError> {
    Ok(APIResponse::<()>::new()
        .with_code(404_i32)
        .with_message("not found".to_string()))
}

async fn health_check() -> Result<APIResponse<()>, api_error::APIError> {
    tracing::info!("receive health check: ok");
    Ok(APIResponse::<()>::new()
        .with_code(200_i32)
        .with_message("ok".to_string()))
}

pub async fn build_router(setting: &Setting) -> Router {
    let connection = get_db_conn(setting.database.uri.clone()).await;

    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    let router = Router::new().nest("/8bd86ee64", route::build_routes());

    let router = match setting.web.compression.unwrap_or(false) {
        true => {
            //  开启压缩后 SSE 数据无法返回  text/event-stream 单独处理不压缩
            let predicate =
                DefaultPredicate::new().and(NotForContentType::new("text/event-stream"));
            router.layer(CompressionLayer::new().compress_when(predicate))
        }
        false => router,
    };
    let state = Arc::new(AppState {
        pool: connection,
        setting: setting.clone(),
    });
    let router = router
        .layer(cors)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        // .layer(ValidateRequestHeaderLayer::custom(VerificationHeaderFields))
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!(
                        "request",
                        method = %method,
                        uri = %uri,
                        matched_path = ?matched_path,
                    )
                })
                .on_failure(()),
        )
        .layer(Extension(state))
        .route_with_tsr("/health/check/", get(health_check).post(health_check));
    router.fallback(handler_404)
}
