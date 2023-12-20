use base64::Engine;
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
                "https://www.bing.com/search",
                // filters=rcrse:"1" makes it not try to autocorrect
                &[("q", query), ("filters", "rcrse:\"1\"")],
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
            .result("#b_results > li.b_algo")
            .title(".b_algo h2 > a")
            .href(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let url = el
                    .select(&Selector::parse("a[href]").unwrap())
                    .next()
                    .and_then(|n| n.value().attr("href"))
                    .unwrap_or_default();
                clean_url(url)
            })))
            .description(".b_caption > p, p.b_algoSlug"),
    )
}

fn clean_url(url: &str) -> eyre::Result<String> {
    // clean up bing's tracking urls
    if url.starts_with("https://www.bing.com/ck/a?") {
        // get the u param
        let url = Url::parse(url)?;
        let u = url
            .query_pairs()
            .find(|(key, _)| key == "u")
            .unwrap_or_default()
            .1;
        // cut off the "a1" and base64 decode
        let u = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&u[2..])
            .unwrap_or_default();
        // convert to utf8
        Ok(String::from_utf8_lossy(&u).to_string())
    } else {
        Ok(url.to_string())
    }
}
