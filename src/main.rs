use axum::routing::{get, Router};
use dotenvy;
use std::fs;
use std::path::Path;

mod handler;
mod route;

#[tokio::main]
async fn main() {
    // init config
    let dotfile = Path::new("clash/.env");

    if !dotfile.exists() {
        let _ = copy_directory(Path::new("default"), Path::new("clash"));
    }

    // load dotfile
    dotenvy::from_path(Path::new("clash/.env")).expect(".env file not found");

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

fn copy_directory<S: AsRef<Path>, D: AsRef<Path>>(
    source: S,
    destination: D,
) -> std::io::Result<()> {
    // 创建目标目录
    fs::create_dir_all(&destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.as_ref().join(entry.file_name());

        if path.is_dir() {
            // 如果是目录，递归复制
            copy_directory(path, dest_path)?;
        } else {
            // 如果是文件，直接复制
            fs::copy(path, dest_path)?;
        }
    }

    Ok(())
}
