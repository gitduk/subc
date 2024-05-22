use axum::{
    extract::{MatchedPath, Request},
    middleware as axum_middleware,
    routing::get,
    Router,
};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::middleware;
use crate::service::*;

pub fn init_app() -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/clash", get(clash::from_url))
        .layer((
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();
                    // axum automatically adds this extension.
                    let path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());
                    tracing::debug_span!("Req", %method, %uri, path)
                })
                .on_failure(()),
            axum_middleware::from_fn(middleware::print_request_body),
            TimeoutLayer::new(std::time::Duration::from_secs(10)),
        ))
}
