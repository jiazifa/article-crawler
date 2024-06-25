use axum::{body::Body, http::Request, response::Response};
use futures::future::BoxFuture;
use std::sync::Arc;
use std::{
    fmt::Display,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};

use crate::AppState;

#[derive(Clone)]
pub struct JwtLayer {
    pub state: Arc<AppState>,
}

impl<S> Layer<S> for JwtLayer {
    type Service = JwtMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JwtMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct JwtMiddleware<S> {
    inner: S,
    state: Arc<AppState>,
}

impl<S> Service<Request<Body>> for JwtMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let is_authorized = matches!(
            super::auth_claim::AuthClaims::extract_from_request(
                request.headers(),
                self.state.setting.jwt.secret.clone(),
            ),
            Ok(claims)
        );

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            response = match is_authorized {
                true => future.await?,
                false => {
                    let (mut parts, _body) = response.into_parts();
                    let json_body = Body::from(
                        serde_json::to_string(
                            &crate::response::APIResponse::<()>::new()
                                .with_code(401_i32)
                                .with_message("Unauthorized".to_string()),
                        )
                        .unwrap(),
                    );
                    parts
                        .headers
                        .insert("content-type", "application/json".parse().unwrap());
                    Response::from_parts(parts, json_body)
                }
            };

            Ok(response)
        })
    }
}
