pub mod search;

use axum::{http::header, routing::get, Router};

pub const BIND_ADDRESS: &str = "[::]:3000";

pub async fn run() {
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    include_str!("assets/index.html"),
                )
            }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("assets/style.css"),
                )
            }),
        )
        .route("/search", get(search::route));

    println!("Listening on {BIND_ADDRESS}");

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
