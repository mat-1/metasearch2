use reqwest::Url;

use crate::{
    engines::{EngineResponse, RequestResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> RequestResponse {
    CLIENT
        .get(
            Url::parse_with_params(
                "https://stract.com/search",
                &[
                    ("ss", "false"),
                    // this is not a tracking parameter or token
                    // this is stract's default value for the search rankings parameter
                    ("sr", "N4IgNglg1gpgJiAXAbQLoBoRwgZ0rBFDEAIzAHsBjApNAXyA"),
                    ("q", query),
                    ("optic", ""),
                ],
            )
            .unwrap(),
        )
        .into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            .result("div.grid.w-full.grid-cols-1.space-y-10.place-self-start > div > div.flex.min-w-0.grow.flex-col")
            .title("a[title]")
            .href("a[href]")
            .description("#snippet-text"),
    )
}
