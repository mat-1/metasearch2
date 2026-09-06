use serde::Deserialize;
use url::Url;

use crate::engines::{EngineResponse, EngineSearchResult, RequestResponse, SearchQuery, CLIENT};

#[derive(Deserialize)]
struct PuriliResponse {
    results: Vec<PuriliResult>,
}

#[derive(Deserialize)]
struct PuriliResult {
    title: String,
    url: String,
    description: String,
}

pub async fn request(query: &SearchQuery) -> RequestResponse {
    let url = Url::parse_with_params("https://puri.li/api/search", &[("q", query.query.as_str())])
        .unwrap();
    CLIENT.get(url).into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let response: PuriliResponse = serde_json::from_str(body)?;
    let search_results = response
        .results
        .into_iter()
        .map(|item| EngineSearchResult {
            title: item.title,
            url: item.url,
            description: item.description,
        })
        .collect();

    Ok(EngineResponse {
        search_results,
        featured_snippet: None,
        answer_html: None,
        infobox_html: None,
    })
}

pub fn request_autocomplete(query: &str) -> wreq::RequestBuilder {
    CLIENT.get(Url::parse_with_params("https://puri.li/api/suggest", &[("q", query)]).unwrap())
}

pub fn parse_autocomplete_response(body: &str) -> eyre::Result<Vec<String>> {
    let res = serde_json::from_str::<Vec<serde_json::Value>>(body)?;
    Ok(res
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect())
}
