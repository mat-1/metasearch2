pub mod autocomplete;
mod image_proxy;
pub mod index;
pub mod opensearch;
pub mod search;

use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use axum::{
    http::header,
    routing::{get, MethodRouter},
    Router,
};
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) {
    let bind_addr = config.bind;

    fn static_route<S>(
        content: &'static str,
        content_type: &'static str,
    ) -> MethodRouter<S, Infallible>
    where
        S: Clone + Send + Sync + 'static,
    {
        let response = ([(header::CONTENT_TYPE, content_type)], content);
        get(|| async { response })
    }

    let app = Router::new()
        .route("/", get(index::index))
        .route(
            "/style.css",
            static_route(include_str!("assets/style.css"), "text/css; charset=utf-8"),
        )
        .route(
            "/script.js",
            static_route(
                include_str!("assets/script.js"),
                "text/javascript; charset=utf-8",
            ),
        )
        .route(
            "/robots.txt",
            static_route(
                include_str!("assets/robots.txt"),
                "text/plain; charset=utf-8",
            ),
        )
        .route(
            "/themes/catppuccin-mocha.css",
            static_route(
                include_str!("assets/themes/catppuccin-mocha.css"),
                "text/css; charset=utf-8",
            ),
        )
        .route("/opensearch.xml", get(opensearch::route))
        .route("/search", get(search::route))
        .route("/autocomplete", get(autocomplete::route))
        .route("/image-proxy", get(image_proxy::route))
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
