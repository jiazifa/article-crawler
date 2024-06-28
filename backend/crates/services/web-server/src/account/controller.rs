use crate::{
    account, api_error::APIError, middlewares::auth_claim::ecode_to_jwt,
    middlewares::auth_claim::AuthClaims, response::APIResponse, AppState,
};
use axum::{
    routing::{get, post},
    Router,
};
use axum::{Extension, Json};
use axum_extra::routing::RouterExt;
use chrono::{DateTime, NaiveDateTime};
use lib_core::auth::{
    schema::{AccountModel, LoginAccountRequest, LoginAccountResponse, RegisterAccountRequest},
    AccountController,
};

use std::sync::Arc;

pub(crate) async fn register_account(
    app: Extension<Arc<AppState>>,
    Json(req): Json<RegisterAccountRequest>,
) -> Result<APIResponse<bool>, APIError> {
    let conn = &app.pool;
    let account_controller = AccountController;
    if let Err(err) = account_controller.register_account(req, conn).await {
        tracing::error!("register account error: {:?}", err);
        return Err(APIError::Internal);
    }
    Ok(APIResponse::<bool>::new()
        .with_code(200_i32)
        .with_data(true))
}

// login account
pub async fn login_account(
    app: Extension<Arc<AppState>>,
    Json(req): Json<LoginAccountRequest>,
) -> Result<APIResponse<LoginAccountResponse>, APIError> {
    let conn = &app.pool;
    let account_controller = AccountController;
    let login_account = account_controller.login_account(req, conn).await?;
    tracing::info!("login account: {:?}", login_account);
    let now = chrono::Utc::now();
    let one_month_after = now + chrono::Duration::days(30);

    let claims = AuthClaims::new(
        None,
        Some("auth".to_string()),
        None,
        one_month_after.timestamp() as usize,
        now.timestamp() as usize,
        now.timestamp() as usize,
        login_account.token,
    );
    let jwt = ecode_to_jwt(&claims, app.setting.jwt.secret.as_bytes());
    let jwt_resp = match jwt {
        Some(j) => j,
        None => return Err(APIError::Internal),
    };
    let new_resp = LoginAccountResponse {
        token: jwt_resp,
        account: login_account.account,
    };
    Ok(APIResponse::<LoginAccountResponse>::new()
        .with_code(200_i32)
        .with_data(new_resp))
}

// account info
pub async fn account_info(
    app: Extension<Arc<AppState>>,
    claims: AuthClaims,
) -> Result<APIResponse<AccountModel>, APIError> {
    let conn = &app.pool;
    let account = claims.get_user(conn).await?;
    Ok(APIResponse::<AccountModel>::new()
        .with_code(200_i32)
        .with_data(account))
}

pub(crate) fn build_routes() -> axum::Router {
    Router::new()
        // 注册用户
        .route_with_tsr("/register", post(register_account))
        .route_with_tsr("/login", post(login_account))
        .route_with_tsr("/info", get(account_info))
}
