use axum::{
    http::{header, HeaderMap},
    response::IntoResponse,
};
use maud::{html, PreEscaped};

pub async fn route(headers: HeaderMap) -> impl IntoResponse {
    let host = headers
        .get("host")
        .and_then(|host| host.to_str().ok())
        .unwrap_or("localhost");

    (
        [(
            header::CONTENT_TYPE,
            "application/opensearchdescription+xml",
        )],
        html! {
            (PreEscaped(r#"<?xml version="1.0" encoding="utf-8"?>"#))
            OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/" {
                ShortName { "metasearch" }
                Description { "Search metasearch" }
                InputEncoding { "UTF-8" }
                Url type="text/html" method="get" template=(format!("https://{host}/search?q={{searchTerms}}")) {}
                Url type="application/x-suggestions+json" method="get" template=(format!("https://{host}/autocomplete?q={{searchTerms}}")) {}
            }
        }.into_string(),
    )
}
