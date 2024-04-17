use reqwest::Url;

use crate::{
    engines::{EngineResponse, RequestResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> RequestResponse {
    CLIENT
        .get(
            Url::parse_with_params(
                "https://scholar.google.com/scholar",
                &[("hl", "en"), ("as_sdt", "0,5"), ("q", query), ("btnG", "")],
            )
            .unwrap(),
        )
        .into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            .result("div.gs_r")
            .title("h3")
            .href("h3 > a[href]")
            .description("div.gs_rs"),
    )
}
