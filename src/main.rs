use axum::routing::{get, Router};
use dotenvy::dotenv;

mod handler;
mod route;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/sub", get(route::sub));

    let app = app.fallback(handler::handler_404);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
