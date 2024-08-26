//! Rendering results in the "all" tab.

use maud::{html, PreEscaped};

use crate::{
    config::Config,
    engines::{self, EngineSearchResult, Infobox, Response},
    web::search::render_engine_list,
};

pub fn render_results(response: Response) -> PreEscaped<String> {
    let mut html = String::new();
    if let Some(infobox) = &response.infobox {
        html.push_str(
            &html! {
                div.infobox {
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
                div.answer {
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

fn render_search_result(
    result: &engines::SearchResult<EngineSearchResult>,
    config: &Config,
) -> PreEscaped<String> {
    let is_ad = result.engines.iter().any(|e| e.id() == "ads");
    html! {
        div.search-result {
            a.search-result-anchor rel="noreferrer" href=(result.result.url) {
                span.search-result-url {
                    @if is_ad {
                        "Ad · "
                    }
                    (result.result.url)
                }
                h3.search-result-title { (result.result.title) }
            }
            p.search-result-description { (result.result.description) }
            (render_engine_list(&result.engines.iter().copied().collect::<Vec<_>>(), config))
        }
    }
}

fn render_featured_snippet(
    featured_snippet: &engines::FeaturedSnippet,
    config: &Config,
) -> PreEscaped<String> {
    html! {
        div.featured-snippet {
            p.search-result-description { (featured_snippet.description) }
            a.search-result-anchor rel="noreferrer" href=(featured_snippet.url) {
                span.search-result-url { (featured_snippet.url) }
                h3.search-result-title { (featured_snippet.title) }
            }
            (render_engine_list(&[featured_snippet.engine], config))
        }
    }
}

pub fn render_infobox(infobox: &Infobox, config: &Config) -> PreEscaped<String> {
    html! {
        div.infobox.postsearch-infobox {
            (infobox.html)
            (render_engine_list(&[infobox.engine], &config))
        }
    }
}
