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
                    ("profile", "default"),
                    ("js", "default"),
                    ("adtech", "default"),
                ],
            )
            .unwrap(),
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
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
