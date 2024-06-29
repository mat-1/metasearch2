mod autocomplete;
mod image_proxy;
mod index;
mod opensearch;
mod search;
mod settings;

use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, MethodRouter},
    Router,
};
use axum_extra::extract::CookieJar;
use maud::{html, Markup, PreEscaped};
use tracing::info;

use crate::config::Config;

pub async fn run(config: Config) {
    let bind_addr = config.bind;

    let config = Arc::new(config);

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
        .route("/", get(index::get))
        .route("/search", get(search::get))
        .route("/settings", get(settings::get))
        .route("/settings", post(settings::post))
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
        .route("/autocomplete", get(autocomplete::route))
        .route("/image-proxy", get(image_proxy::route))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            config_middleware,
        ))
        .with_state(config);

    info!("Listening on http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn config_middleware(
    State(config): State<Arc<Config>>,
    cookies: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut config = config.clone().as_ref().clone();

    fn set_from_cookie(config: &mut String, cookies: &CookieJar, name: &str) {
        if let Some(cookie) = cookies.get(name) {
            let value = cookie.value();
            *config = value.to_string();
        }
    }

    set_from_cookie(&mut config.ui.stylesheet_url, &cookies, "stylesheet-url");
    set_from_cookie(&mut config.ui.stylesheet_str, &cookies, "stylesheet-str");

    // modify the state
    req.extensions_mut().insert(config);

    Ok(next.run(req).await)
}

pub fn head_html(title: Option<&str>, config: &Config) -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title {
                @if let Some(title) = title {
                    { (title) }
                    { " - " }
                }
                {(config.ui.site_name)}
            }
            link rel="stylesheet" href="/style.css";
            @if !config.ui.stylesheet_url.is_empty() {
                link rel="stylesheet" href=(config.ui.stylesheet_url);
            }
            @if !config.ui.stylesheet_str.is_empty() {
                style { (PreEscaped(html_escape::encode_style(&config.ui.stylesheet_str))) }
            }
            script src="/script.js" defer {}
            link rel="search" type="application/opensearchdescription+xml" title="metasearch" href="/opensearch.xml";
        }
    }
}
