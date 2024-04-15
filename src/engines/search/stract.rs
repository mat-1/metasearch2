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
            .title("div.flex.min-w-0 > div.flex.min-w-0.grow.flex-col")
            .href("a[href]")
            .description("div.text-sm.font-normal.text-neutral-focus > div.snippet > div > div > span#snippet-text.snippet-text > span"),
    )
}
