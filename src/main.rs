mod handler;
mod middleware;
mod route;
mod service;
mod structs;
mod utils;

use handler::*;
use std::time::Duration;
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "subc=debug,tower_http=debug,axum=debug".into()),
        )
        .with(fmt::layer())
        .init();

    let app = crate::route::init_app();
    let app = app.fallback(handler_404);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3078").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
