use reqwest::Url;
use scraper::{ElementRef, Selector};

use crate::{
    engines::EngineResponse,
    parse::{parse_html_response_with_opts, ParseOpts, QueryMethod},
};

pub fn request(client: &reqwest::Client, query: &str) -> reqwest::RequestBuilder {
    client
        .get(
            Url::parse_with_params(
                "https://www.google.com/search",
                // nfpr makes it not try to autocorrect
                &[("q", query), ("nfpr", "1")],
            )
            .unwrap(),
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            // xpd is weird, some results have it but it's usually used for ads?
            // the :first-child filters out the ads though since for ads the first child is always a span
            .result("div.g > div, div.xpd > div:first-child")
            .title("h3")
            .href("a[href]")
            .description("div[data-sncf], div[style='-webkit-line-clamp:2']")
            .featured_snippet("block-component")
            .featured_snippet_description("div[data-attrid='wa:/description'] > span:first-child")
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

pub fn request_autocomplete(client: &reqwest::Client, query: &str) -> reqwest::RequestBuilder {
    client
        .get(
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
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
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
