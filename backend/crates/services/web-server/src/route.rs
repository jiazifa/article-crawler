use axum::{
    routing::{get, post},
    Router,
};
use axum_extra::routing::RouterExt;

use crate::{account, feed};

pub fn build_routes() -> Router {
    Router::new()
        .nest("/rss", feed::controller::build_routes())
        .nest("/account", account::controller::build_routes())
}
