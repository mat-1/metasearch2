use reqwest::Url;

use crate::{
    engines::{EngineResponse, RequestResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> RequestResponse {
    CLIENT
        .get(Url::parse_with_params("https://rightdao.com/search", &[("q", query)]).unwrap())
        .into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            .result("div.item")
            .title("div.title")
            .href("a[href]")
            .description("div.description"),
    )
}
