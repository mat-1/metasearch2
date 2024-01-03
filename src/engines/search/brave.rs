use url::Url;

use crate::{
    engines::{EngineResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(Url::parse_with_params("https://search.brave.com/search", &[("q", query)]).unwrap())
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
