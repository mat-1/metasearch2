pub mod index;
pub mod search;
pub mod style_css;

use axum::{routing::get, Router};

pub async fn run() {
    let app = Router::new()
        .route("/", get(index::route))
        .route("/style.css", get(style_css::route))
        .route("/search", get(search::route));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
