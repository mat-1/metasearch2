pub mod autocomplete;
pub mod opensearch;
pub mod search;

use std::{net::SocketAddr, sync::Arc};

use axum::{http::header, routing::get, Router};
use tracing::info;

use crate::config::Config;

const BASE_COMMIT_URL: &str = "https://github.com/mat-1/metasearch2/commit/";
const VERSION: &str = std::env!("CARGO_PKG_VERSION");
const COMMIT_HASH: &str = std::env!("GIT_HASH");
const COMMIT_HASH_SHORT: &str = std::env!("GIT_HASH_SHORT");

pub async fn run(config: Config) {
    let bind_addr = config.bind;

    let version_info = if config.version_info.unwrap() {
        if COMMIT_HASH == "unknown" || COMMIT_HASH_SHORT == "unknown" {
            format!(r#"<span class="version-info">Version {VERSION} (unknown commit)</span>"#)
        } else {
            format!(
                r#"<span class="version-info">Version {VERSION} (<a href="{BASE_COMMIT_URL}{COMMIT_HASH}">{COMMIT_HASH_SHORT}</a>)</span>"#
            )
        }
    } else {
        String::new()
    };

    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    include_str!("assets/index.html").replace("%version-info%", &version_info),
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
        .route("/search", get(search::route))
        .route("/autocomplete", get(autocomplete::route))
        .with_state(Arc::new(config));

    info!("Listening on {bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
