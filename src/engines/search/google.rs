use scraper::{ElementRef, Selector};
use url::Url;

use crate::{
    engines::{EngineResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts, QueryMethod},
};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://www.google.com/search",
            // nfpr makes it not try to autocorrect
            &[("q", query), ("nfpr", "1")],
        )
        .unwrap(),
    )
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            // xpd is weird, some results have it but it's usually used for ads?
            // the :first-child filters out the ads though since for ads the first child is always a
            // span
            .result("div.g > div, div.xpd > div:first-child")
            .title("h3")
            .href("a[href]")
            .description("div[data-sncf='2'], div[style='-webkit-line-clamp:2']")
            .featured_snippet("block-component")
            .featured_snippet_description(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let Some(description_container_el) = el
                    .select(
                        &Selector::parse("div[data-attrid='wa:/description'] > span:first-child")
                            .unwrap(),
                    )
                    .next()
                else {
                    return Ok(String::new());
                };

                // build the description
                let mut description = String::new();
                iter_featured_snippet_children(&mut description, &description_container_el);

                Ok(description)
            })))
            .featured_snippet_title("h3")
            .featured_snippet_href(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let url = el
                    .select(&Selector::parse("a").unwrap())
                    .next()
                    .and_then(|n| n.value().attr("href"))
                    .unwrap_or_default();
                clean_url(url)
            }))),
    )
}

// Google autocomplete responses sometimes include clickable links that include
// text that we shouldn't show.
// We can filter for these by removing any elements matching
// [data-ved]:not([data-send-open-event])
fn iter_featured_snippet_children(description: &mut String, el: &ElementRef) {
    for inner_node in el.children() {
        match inner_node.value() {
            scraper::Node::Text(t) => {
                description.push_str(&t.text);
            }
            scraper::Node::Element(inner_el) => {
                if inner_el.attr("data-ved").is_none()
                    || inner_el.attr("data-send-open-event").is_some()
                {
                    iter_featured_snippet_children(
                        description,
                        &ElementRef::wrap(inner_node).unwrap(),
                    );
                }
            }
            _ => {}
        }
    }
}

pub fn request_autocomplete(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://suggestqueries.google.com/complete/search",
            &[
                ("output", "firefox"),
                ("client", "firefox"),
                ("hl", "US-en"),
                ("q", query),
            ],
        )
        .unwrap(),
    )
}

pub fn parse_autocomplete_response(body: &str) -> eyre::Result<Vec<String>> {
    let res = serde_json::from_str::<Vec<serde_json::Value>>(body)?;
    Ok(res
        .into_iter()
        .nth(1)
        .unwrap_or_default()
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect())
}

fn clean_url(url: &str) -> eyre::Result<String> {
    if url.starts_with("/url?q=") {
        // get the q param
        let url = Url::parse(format!("https://www.google.com{url}").as_str())?;
        let q = url
            .query_pairs()
            .find(|(key, _)| key == "q")
            .unwrap_or_default()
            .1;
        Ok(q.to_string())
    } else {
        Ok(url.to_string())
    }
}
