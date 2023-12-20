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

use crate::engines::{self, ProgressUpdate, ProgressUpdateKind, Response};

fn render_beginning_of_html(query: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - metasearch</title>
    <link rel="stylesheet" href="/style.css">
    <script src="/script.js" defer></script>
</head>
<body>
    <main>
    <form action="/search" method="get" class="search-form">
        <input type="text" name="q" placeholder="Search" value="{}" id="search-input" autofocus onfocus="this.select()" autocomplete="off">
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

fn render_engine_list(engines: &[engines::Engine]) -> String {
    let mut html = String::new();
    for engine in engines {
        html.push_str(&format!(
            r#"<span class="engine-list-item">{engine}</span>"#,
            engine = encode_text(&engine.id())
        ));
    }
    format!(r#"<div class="engine-list">{html}</div>"#)
}

fn render_search_result(result: &engines::SearchResult) -> String {
    format!(
        r#"<div class="search-result">
    <a class="search-result-anchor" href="{url_attr}">
        <span class="search-result-url" href="{url_attr}">{url}</span>
        <h3 class="search-result-title">{title}</h3>
    </a>
    <p class="search-result-description">{desc}</p>
    {engines_html}
    </div>
"#,
        url_attr = encode_unquoted_attribute(&result.url),
        url = encode_text(&result.url),
        title = encode_text(&result.title),
        desc = encode_text(&result.description),
        engines_html = render_engine_list(&result.engines.iter().copied().collect::<Vec<_>>())
    )
}

fn render_featured_snippet(featured_snippet: &engines::FeaturedSnippet) -> String {
    format!(
        r#"<div class="featured-snippet">
    <p class="search-result-description">{desc}</p>
    <a class="search-result-anchor" href="{url_attr}">
        <span class="search-result-url" href="{url_attr}">{url}</span>
        <h3 class="search-result-title">{title}</h3>
    </a>
    {engines_html}
    </div>
"#,
        desc = encode_text(&featured_snippet.description),
        url_attr = encode_unquoted_attribute(&featured_snippet.url),
        url = encode_text(&featured_snippet.url),
        title = encode_text(&featured_snippet.title),
        engines_html = render_engine_list(&[featured_snippet.engine])
    )
}

fn render_results(response: Response) -> String {
    let mut html = String::new();
    if let Some(featured_snippet) = response.featured_snippet {
        html.push_str(&render_featured_snippet(&featured_snippet));
    }
    for result in &response.search_results {
        html.push_str(&render_search_result(result));
    }
    html
}

fn render_progress_update(progress_update: &ProgressUpdate) -> String {
    let message: &str = match progress_update.kind {
        ProgressUpdateKind::Requesting => "requesting",
        ProgressUpdateKind::Downloading => "downloading",
        ProgressUpdateKind::Parsing => "parsing",
        ProgressUpdateKind::Done => "<span class=\"progress-update-done\">done</span>",
    };

    format!(
        r#"<span class="progress-update-time">{time:>4}ms</span> {engine} {message}"#,
        time = progress_update.time,
        message = message,
        engine = progress_update.engine.id()
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
                render_progress_update(&progress_update)
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
        second_half.push_str(&render_results(results));
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
