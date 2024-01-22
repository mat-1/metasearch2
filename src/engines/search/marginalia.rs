use reqwest::Url;

use crate::{
    engines::{EngineResponse, RequestResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts},
};

pub fn request(query: &str) -> RequestResponse {
    // if the query is more than 3 words or has any special characters then abort
    if query.split_whitespace().count() > 3
        || !query.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ')
    {
        return RequestResponse::None;
    }

    CLIENT
        .get(
            Url::parse_with_params(
                "https://search.marginalia.nu/search",
                &[
                    ("query", query),
                    ("profile", "corpo"),
                    ("js", "default"),
                    ("adtech", "default"),
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
            .result("section.search-result")
            .title("h2")
            .href("a[href]")
            .description("p.description"),
    )
}
