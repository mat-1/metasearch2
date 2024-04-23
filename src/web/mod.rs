pub mod autocomplete;
pub mod captcha;
pub mod index;
pub mod opensearch;
pub mod search;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::header,
    routing::{get, post},
    Router,
};
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) {
    let bind_addr = config.bind;

    let app = Router::new()
        .route("/", get(index::index))
        .route("/captcha", get(captcha::get))
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("assets/style.css"),
                )
            }),
        )
        .route(
            "/script.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("assets/script.js"),
                )
            }),
        )
        .route(
            "/robots.txt",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    include_str!("assets/robots.txt"),
                )
            }),
        )
        .route("/opensearch.xml", get(opensearch::route))
        .route("/search", get(captcha::get))
        .route("/search", post(search::post))
        .route("/autocomplete", get(autocomplete::route))
        .with_state(Arc::new(config));

    info!("Listening on http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
