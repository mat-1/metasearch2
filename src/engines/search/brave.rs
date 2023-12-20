use reqwest::Url;

use crate::{
    engines::EngineResponse,
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(client: &reqwest::Client, query: &str) -> reqwest::RequestBuilder {
    client
        .get(Url::parse_with_params("https://search.brave.com/search", &[("q", query)]).unwrap())
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
            .result("#results > .snippet[data-pos]:not(.standalone)")
            .title(".url")
            .href("a")
            .description(".snippet-content, .video-snippet > .snippet-description"),
    )
}
