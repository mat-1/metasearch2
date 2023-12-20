use std::collections::HashMap;

use async_stream::stream;
use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use html_escape::{encode_text, encode_unquoted_attribute};

use crate::engines;

fn render_beginning_of_html(query: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - metasearch</title>
    <link rel="stylesheet" href="/style.css">
</head>
<body>
    <main>
    <form action="/search" method="get" class="search-form">
        <input type="text" name="q" placeholder="Search" value="{}">
        <input type="submit" value="Search">
    </form>
    <div class="progress-updates">
"#,
        encode_text(query),
        encode_unquoted_attribute(query)
    )
}

fn render_end_of_html() -> String {
    r#"</main></body></html>"#.to_string()
}

fn render_search_result(result: &engines::SearchResult) -> String {
    let engines_html = result
        .engines
        .iter()
        .map(|engine| {
            format!(
                r#"<span class="search-result-engines-item">{}</span>"#,
                encode_text(&engine.name())
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<div class="search-result">
    <a class="search-result-anchor" href="{url_attr}">
        <span class="search-result-url" href="{url_attr}">{url}</span>
        <h3 class="search-result-title">{title}</h3>
    </a>
    <p class="search-result-description">{desc}</p>
    <div class="search-result-engines">{engines_html}</div>
    </div>
"#,
        url_attr = encode_unquoted_attribute(&result.url),
        url = encode_text(&result.url),
        title = encode_text(&result.title),
        desc = encode_text(&result.description)
    )
}

pub async fn route(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let query = params
        .get("q")
        .cloned()
        .unwrap_or_default()
        .trim()
        .replace('\n', " ");
    if query.is_empty() {
        // redirect to index
        return (
            StatusCode::FOUND,
            [
                (header::LOCATION, "/"),
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            ],
            Body::from("<a href=\"/\">No query provided, click here to go back to index</a>"),
        );
    }

    let s = stream! {
        type R = Result<Bytes, eyre::Error>;

        yield R::Ok(Bytes::from(render_beginning_of_html(&query)));

        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

        let search_future = tokio::spawn(async move { engines::search(&query, progress_tx).await });

        while let Some(progress_update) = progress_rx.recv().await {
            let progress_html = format!(
                r#"<p class="progress-update">{}</p>"#,
                encode_text(&progress_update.to_string())
            );
            yield R::Ok(Bytes::from(progress_html));
        }

        let results = match search_future.await? {
            Ok(results) => results,
            Err(e) => {
                let error_html = format!(
                    r#"<h1>Error: {}</p>"#,
                    encode_text(&e.to_string())
                );
                yield R::Ok(Bytes::from(error_html));
                return;
            }
        };

        let mut second_half = String::new();

        second_half.push_str("</div>"); // close progress-updates
        second_half.push_str("<style>.progress-updates{display:none}</style>");
        for result in results.search_results {
            second_half.push_str(&render_search_result(&result));
        }
        second_half.push_str(&render_end_of_html());

        yield Ok(Bytes::from(second_half));

    };

    let stream = Body::from_stream(s);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::TRANSFER_ENCODING, "chunked"),
        ],
        stream,
    )
}
