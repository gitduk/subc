use axum::{
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, Router},
};
use std::path::Path;
use std::time::Duration;
use tokio::signal;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod middlewares;
mod route;

#[tokio::main]
async fn main() {
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "subc=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    // init dotfile
    let dotfile = Path::new("clash/.env");

    if !dotfile.exists() {
        match std::fs::File::create(dotfile) {
            Ok(_) => {
                tracing::debug!("create .env");
            },
            Err(e) => {
                eprint!("create .env failed: {e}");
            }
        }
    }

    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/sub", get(route::sub))
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

                    tracing::debug_span!("request", %method, %uri, path)
                })
                .on_failure(()),
            middleware::from_fn(middlewares::print_request_body),
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    let app = app.fallback(handler_404);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
