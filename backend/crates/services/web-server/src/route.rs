use axum::{
    routing::{get, post},
    Router,
};
use axum_extra::routing::RouterExt;

use crate::{account, rss};

pub fn build_routes() -> Router {
    Router::new()
        .nest("/rss", rss::controller::build_routes())
        .nest("/account", account::controller::build_routes())
}
