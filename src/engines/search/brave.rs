use url::Url;

use crate::{
    engines::{EngineResponse, RequestResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> RequestResponse {
    // brave search doesn't support exact matching anymore, so disable it to not
    // pollute the results
    if query.chars().any(|c| c == '"') {
        return RequestResponse::None;
    }

    CLIENT
        .get(Url::parse_with_params("https://search.brave.com/search", &[("q", query)]).unwrap())
        .into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            .result("#results > .snippet[data-pos]:not(.standalone)")
            .title(".title")
            .href("a")
            .description(".snippet-content, .video-snippet > .snippet-description"),
    )
}
