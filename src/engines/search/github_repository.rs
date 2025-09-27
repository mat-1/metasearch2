use serde_json::Value;

use crate::engines::{EngineResponse, EngineSearchResult, CLIENT};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    let url = format!(
        "https://api.github.com/search/repositories?sort=stars&order=desc&q={}",
        urlencoding::encode(query)
    );
    CLIENT
        .get(url)
        .header("Accept", "application/vnd.github.preview.text-match+json")
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let json: Value = serde_json::from_str(body)?;
    let items = json["items"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut search_results = Vec::new();

    for item in items {
        let full_name = item["full_name"].as_str().unwrap_or("");
        let language = item["language"].as_str().unwrap_or("");
        let description = item["description"].as_str().unwrap_or("");
        let content_parts: Vec<&str> = vec![language, description]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        let content = content_parts.join(" / ");
        let url = item["html_url"].as_str().unwrap_or("");

        search_results.push(EngineSearchResult {
            url: url.to_string(),
            title: full_name.to_string(),
            description: content,
        });
    }

    Ok(EngineResponse {
        search_results,
        ..Default::default()
    })
}
