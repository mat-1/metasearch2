use axum::{http::header, response::IntoResponse, Extension};
use maud::{html, PreEscaped, DOCTYPE};

use crate::{config::Config, web::head_html};

const BASE_COMMIT_URL: &str = "https://github.com/mat-1/metasearch2/commit/";
const VERSION: &str = std::env!("CARGO_PKG_VERSION");
const COMMIT_HASH: &str = std::env!("GIT_HASH");
const COMMIT_HASH_SHORT: &str = std::env!("GIT_HASH_SHORT");

pub async fn get(Extension(config): Extension<Config>) -> impl IntoResponse {
    let html = html! {
        (PreEscaped("<!-- source code: https://github.com/mat-1/metasearch2 -->\n"))
        (DOCTYPE)
        html lang="en" {
            {(head_html(None, &config))}
            body {
                @if config.ui.show_settings_link {
                    a.settings-link href="/settings" { "Settings" }
                }
                div.main-container.index-page {
                    h1 { {(config.ui.site_name)} }
                    form.search-form action="/search" method="get" {
                        input type="text" name="q" placeholder="Search" id="search-input" autofocus onfocus="this.select()" autocomplete="off";
                        input type="submit" value="Search";
                    }
                }
                @if config.ui.show_version_info {
                    span.version-info {
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
    .into_string();

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}
