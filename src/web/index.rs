use std::sync::Arc;

use axum::{extract::State, http::header, response::IntoResponse};
use maud::{html, PreEscaped, DOCTYPE};

use crate::config::Config;

const BASE_COMMIT_URL: &str = "https://github.com/mat-1/metasearch2/commit/";
const VERSION: &str = std::env!("CARGO_PKG_VERSION");
const COMMIT_HASH: &str = std::env!("GIT_HASH");
const COMMIT_HASH_SHORT: &str = std::env!("GIT_HASH_SHORT");

pub async fn index(State(config): State<Arc<Config>>) -> impl IntoResponse {
    let mut html = String::new();
    html.push_str(
        &html! {
            (PreEscaped("<!-- source code: https://github.com/mat-1/metasearch2 -->\n"))
            (DOCTYPE)
            html lang="en" {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { "metasearch" }
                    link rel="stylesheet" href="/style.css";
                    @if let Some(stylesheet_url) = &config.ui.stylesheet_url {
                        link rel="stylesheet" href=(stylesheet_url);
                    }
                    @if let Some(stylesheet_str) = &config.ui.stylesheet_str {
                        link rel="stylesheet" href=(stylesheet_str);
                    }
                    script src="/script.js" defer {}
                    link rel="search" type="application/opensearchdescription+xml" title="metasearch" href="/opensearch.xml";
                }
                body {
                    div."main-container" {
                        h1 { "metasearch" }
                        form."search-form" action="/search" method="get" {
                            input type="text" name="q" placeholder="Search" id="search-input" autofocus onfocus="this.select()" autocomplete="off";
                            input type="submit" value="Search";
                        }
                    }
                    @if config.ui.show_version_info {
                        span."version-info" {
                            @if COMMIT_HASH == "unknown" || COMMIT_HASH_SHORT == "unknown" {
                                "Version "
                                (VERSION)
                            } else {
                                "Version "
                                (VERSION)
                                " ("
                                a href=(format!("{BASE_COMMIT_URL}{COMMIT_HASH}")) { (COMMIT_HASH_SHORT) }
                                ")"
                            }
                        }
                    }
                }
            
            }
        }
        .into_string(),
    );

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}
