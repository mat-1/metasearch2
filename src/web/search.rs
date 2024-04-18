use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_stream::stream;
use axum::{
    body::Body,
    extract::{ConnectInfo, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use maud::{html, PreEscaped};

use crate::{
    config::Config,
    engines::{self, Engine, EngineProgressUpdate, ProgressUpdateData, Response, SearchQuery},
};

fn render_beginning_of_html(query: &str) -> String {
    let head_html = html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title {
                (query)
                " - metasearch"
            }
            link rel="stylesheet" href="/style.css";
            script src="/script.js" defer {}
            link rel="search" type="application/opensearchdescription+xml" title="metasearch" href="/opensearch.xml";
        }
    }.into_string();
    let form_html = html! {
        form."search-form" action="/search" method="get" {
            input #"search-input"  type="text" name="q" placeholder="Search" value=(query) autofocus onfocus="this.select()" autocomplete="off";
            input type="submit" value="Search";
        }
    }.into_string();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head_html}
<body>
    <div class="results-container">
    <main>
    {form_html}
    <div class="progress-updates">
"#
    )
}

fn render_end_of_html() -> String {
    r"</main></div></body></html>".to_string()
}

fn render_engine_list(engines: &[engines::Engine], config: &Config) -> PreEscaped<String> {
    let mut html = String::new();
    for (i, engine) in engines.iter().enumerate() {
        if config.ui.show_engine_list_separator.unwrap() && i > 0 {
            html.push_str(" &middot; ");
        }
        let raw_engine_id = &engine.id();
        let engine_id = if config.ui.show_engine_list_separator.unwrap() {
            raw_engine_id.replace('_', " ")
        } else {
            raw_engine_id.to_string()
        };
        html.push_str(&html! { span."engine-list-item" { (engine_id) } }.into_string())
    }
    html! {
        div."engine-list" {
            (PreEscaped(html))
        }
    }
}

fn render_search_result(result: &engines::SearchResult, config: &Config) -> PreEscaped<String> {
    html! {
        div."search-result" {
            a."search-result-anchor" rel="noreferrer" href=(result.url) {
                span."search-result-url" { (result.url) }
                h3."search-result-title" { (result.title) }
            }
            p."search-result-description" { (result.description) }
            (render_engine_list(&result.engines.iter().copied().collect::<Vec<_>>(), config))
        }
    }
}

fn render_featured_snippet(
    featured_snippet: &engines::FeaturedSnippet,
    config: &Config,
) -> PreEscaped<String> {
    html! {
        div."featured-snippet" {
            p."search-result-description" { (featured_snippet.description) }
            a."search-result-anchor" rel="noreferrer" href=(featured_snippet.url) {
                span."search-result-url" { (featured_snippet.url) }
                h3."search-result-title" { (featured_snippet.title) }
            }
            (render_engine_list(&[featured_snippet.engine], config))
        }
    }
}

fn render_results(response: Response) -> PreEscaped<String> {
    let mut html = String::new();
    if let Some(infobox) = &response.infobox {
        html.push_str(
            &html! {
                div."infobox" {
                    (infobox.html)
                    (render_engine_list(&[infobox.engine], &response.config))
                }
            }
            .into_string(),
        );
    }
    if let Some(answer) = &response.answer {
        html.push_str(
            &html! {
                div."answer" {
                    (answer.html)
                    (render_engine_list(&[answer.engine], &response.config))
                }
            }
            .into_string(),
        );
    }
    if let Some(featured_snippet) = &response.featured_snippet {
        html.push_str(&render_featured_snippet(featured_snippet, &response.config).into_string());
    }
    for result in &response.search_results {
        html.push_str(&render_search_result(result, &response.config).into_string());
    }

    if html.is_empty() {
        html.push_str(
            &html! {
                p { "No results." }
            }
            .into_string(),
        );
    }

    PreEscaped(html)
}

fn render_engine_progress_update(
    engine: Engine,
    progress_update: &EngineProgressUpdate,
    time_ms: u64,
) -> String {
    let message = match progress_update {
        EngineProgressUpdate::Requesting => "requesting",
        EngineProgressUpdate::Downloading => "downloading",
        EngineProgressUpdate::Parsing => "parsing",
        EngineProgressUpdate::Done => {
            &{ html! { span."progress-update-done" { "done" } }.into_string() }
        }
    };

    html! {
        span."progress-update-time" {
            (format!("{time_ms:>4}"))
            "ms"
        }
        " "
        (engine)
        " "
        (PreEscaped(message))
    }
    .into_string()
}

pub async fn route(
    Query(params): Query<HashMap<String, String>>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
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

    let query = SearchQuery {
        query,
        request_headers: headers
            .clone()
            .into_iter()
            .map(|(k, v)| {
                (
                    k.map(|k| k.to_string()).unwrap_or_default(),
                    v.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect(),
        ip: headers
            // this could be exploited under some setups, but the ip is only used for the
            // "what is my ip" answer so it doesn't really matter
            .get("x-forwarded-for")
            .map_or_else(
                || addr.ip().to_string(),
                |ip| ip.to_str().unwrap_or_default().to_string(),
            ),
        config: config.clone(),
    };

    let s = stream! {
        type R = Result<Bytes, eyre::Error>;

        // the html is sent in three chunks (technically more if you count progress updates):
        // 1) the beginning of the html, including the search bar
        // 1.5) the progress updates
        // 2) the results
        // 3) the post-search infobox (usually not sent) + the end of the html

        let first_part = render_beginning_of_html(&query);
        // second part is in the loop
        let mut third_part = String::new();

        yield R::Ok(Bytes::from(first_part));

        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

        let search_future = tokio::spawn(async move { engines::search(&query, progress_tx).await });

        while let Some(progress_update) = progress_rx.recv().await {
            match progress_update.data {
                ProgressUpdateData::Engine { engine, update } => {
                    let progress_html = format!(
                        r#"<p class="progress-update">{}</p>"#,
                        render_engine_progress_update(engine, &update, progress_update.time_ms)
                    );
                    yield R::Ok(Bytes::from(progress_html));
                },
                ProgressUpdateData::Response(results) => {
                    let mut second_part = String::new();

                    second_part.push_str("</div>"); // close progress-updates
                    second_part.push_str("<style>.progress-updates{display:none}</style>");
                    second_part.push_str(&render_results(results).into_string());
                    yield Ok(Bytes::from(second_part));
                },
                ProgressUpdateData::PostSearchInfobox(infobox) => {
                    third_part.push_str(&html! {
                        div."infobox"."postsearch-infobox" {
                            (infobox.html)
                            (render_engine_list(&[infobox.engine], &config))
                        }
                    }.into_string());
                }
            }
        }

        if let Err(e) = search_future.await? {
            let error_html = html! {
                h1 {
                    "Error: "
                    (e)
                }
            }.into_string();
            yield R::Ok(Bytes::from(error_html));
            return;
        };

        third_part.push_str(&render_end_of_html());

        yield Ok(Bytes::from(third_part));

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
