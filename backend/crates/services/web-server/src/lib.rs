use axum_extra::routing::RouterExt;
use lib_core::{get_db_conn, DBConnection};
// use middlewares::verification::VerificationHeaderFields;
// use middlewares::VerificationHeaderFields;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{MatchedPath, Request},
    http::{Method, StatusCode, Uri},
    routing::get,
    BoxError, Extension, Router,
};
use response::APIResponse;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::{
    compression::{predicate::NotForContentType, CompressionLayer, DefaultPredicate, Predicate},
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};

mod account;
mod api_error;
mod feed;
mod middlewares;
mod response;
mod route;
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

async fn handle_timeout_error(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    let error_message = format!("`{method} {uri}` failed with {err}");
    tracing::error!("{}", error_message);
    (StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string())
}

pub async fn build_router(setting: &Setting) -> Router {
    let connection = get_db_conn(setting.database.uri.clone()).await;

    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    let router = route::build_routes();

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
        .layer(HandleErrorLayer::new(|err| async move {
            let error_message = format!("unhandled error: {:?}", err);
            tracing::error!("got an error: {}", error_message);

            let response = axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from("Internal Server Error"))
                .unwrap_or_default();
            response
        }))
        .layer(cors)
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        )
        .layer(Extension(state))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(30)),
        )
        .route_with_tsr("/health/check/", get(health_check).post(health_check));
    // router.fallback(handler_404)
    router
}
