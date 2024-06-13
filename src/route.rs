use axum::{
    extract::{MatchedPath, Request},
    routing::get,
    Router,
};
use clash::get_nodes_from;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::service::*;
use crate::structs::AppState;

pub async fn init_app() -> Router {
    let url = std::env::var("URL").expect("URL not set");

    if let Ok(nodes) = get_nodes_from(&url).await {
        let state = AppState { url, nodes };
        Router::new()
            .route("/", get(|| async { "ok" }))
            .route("/clash", get(clash::from_url))
            .route("/provider", get(clash::build_provider))
            .route("/refresh", get(clash::refresh))
            .with_state(state)
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
                TimeoutLayer::new(std::time::Duration::from_secs(10)),
            ))
    } else {
        panic!("Can not get nodes form {url}")
    }
}
