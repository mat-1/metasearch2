use std::{collections::HashMap, net::SocketAddr, };

use axum::{extract::{ConnectInfo, Query, }, http::{header, HeaderMap}, response::IntoResponse, Extension, Form};
use maud::{html, PreEscaped, DOCTYPE};

use crate::{config::Config, web::head_html};

use super::search;

pub async fn get(
    Query(params): Query<HashMap<String, String>>,
    Extension(config): Extension<Config>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<serde_json::Value>,
) -> axum::response::Response {
    let mut html = String::new();

    let Some(captcha_config) = &config.captcha else {
        return search::post(Query(params), Extension(config), headers, ConnectInfo(addr), Form(form)).await;
    };

    html.push_str(
        &html! {
            (DOCTYPE)
            html lang="en" {
                {(head_html(Some("Verify your humanity"), &config))}
                body {
                    script src="https://www.google.com/recaptcha/api.js" async defer {}
                    script {
                        (PreEscaped(r#"
                        function submitCaptcha(token) {
                            const form = document.getElementById("captcha-form");

                            const url = new URL(window.location.href);
                            const q = url.searchParams.get("q");
                            if (q) {
                                form.action = form.action + "?q=" + encodeURIComponent(q);
                            }
                            form.submit();
                        }
                        "#))
                    }
                    style {
                        (PreEscaped(r#"
                        .g-recaptcha {
                            margin: 0 auto;
                            width: fit-content;
                        }
                        "#))
                    }
                    noscript {
                        "You must enable JavaScript to use this site."
                    }
                    div.main-container.index-page {
                        h1 { "Verify your humanity" }
                        form #captcha-form action="/search" method="post" {
                            div."g-recaptcha" data-sitekey=(captcha_config.site_key) data-theme="dark" data-callback="submitCaptcha" {}
                        }
                    }
                }
            
            }
        }
        .into_string(),
    );

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

pub async fn verify(token: &str, secret: &str) -> eyre::Result<bool> {
    let response = reqwest::get(&format!(
        "https://www.google.com/recaptcha/api/siteverify?secret={secret}&response={token}",
        secret = secret,
        token = token
    ))
    .await?
    .json::<serde_json::Value>()
    .await?;

    response
        .get("success")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| eyre::eyre!("Invalid response from Google"))
}
