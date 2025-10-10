use eyre::eyre;
use scraper::Selector;
use serde::Deserialize;
use tracing::error;
use tracing::warn;
use url::Url;

use crate::engines::{
    Engine, EngineImageResult, EngineImagesResponse, EngineResponse, EngineSearchResult,
    RequestResponse, SearchQuery, CLIENT,
};

#[derive(Deserialize)]
pub struct GoogleConfig {
    pub custom_search_api_key: String,
}

#[derive(Deserialize)]
struct CustomSearchResponse {
    items: Option<Vec<CustomSearchItem>>,
}

#[derive(Deserialize)]
struct CustomSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
}

pub fn request(query: &SearchQuery) -> RequestResponse {
    let config_toml = query.config.engines.get(Engine::Google).extra.clone();
    let config: GoogleConfig = match toml::Value::Table(config_toml).try_into() {
        Ok(args) => args,
        Err(err) => {
            error!("Failed to parse Google config: {err}");
            return RequestResponse::None;
        }
    };

    let url = Url::parse_with_params(
        "https://www.googleapis.com/customsearch/v1",
        &[
            ("key", config.custom_search_api_key.as_str()),
            ("cx", "d4e68b99b876541f0"), // https://git.lolcat.ca/lolcat/4get/src/commit/73f8472eeca07250dea9ea789e612f4a27d4ce5c/data/config.php#L175
            ("q", query.query.as_ref()),
            ("num", "10"),
        ],
    )
    .unwrap();
    CLIENT.get(url).into()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let response: CustomSearchResponse = serde_json::from_str(body)?;
    let search_results = response
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|item| EngineSearchResult {
            title: item.title,
            url: item.link,
            description: item.snippet.unwrap_or_default(),
        })
        .collect();

    Ok(EngineResponse {
        search_results,
        featured_snippet: None,
        answer_html: None,
        infobox_html: None,
    })
}

pub fn request_autocomplete(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://suggestqueries.google.com/complete/search",
            &[
                ("output", "firefox"),
                ("client", "firefox"),
                ("hl", "US-en"),
                ("q", query),
            ],
        )
        .unwrap(),
    )
}

pub fn parse_autocomplete_response(body: &str) -> eyre::Result<Vec<String>> {
    let res = serde_json::from_str::<Vec<serde_json::Value>>(body)?;
    Ok(res
        .into_iter()
        .nth(1)
        .unwrap_or_default()
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect())
}

pub fn request_images(query: &str) -> reqwest::RequestBuilder {
    // ok so google also has a json api for images BUT it gives us less results
    CLIENT.get(
        Url::parse_with_params(
            "https://www.google.com/search",
            &[("q", query), ("udm", "2"), ("prmd", "ivsnmbtz")],
        )
        .unwrap(),
    )
}

pub fn parse_images_response(body: &str) -> eyre::Result<EngineImagesResponse> {
    // we can't just scrape the html because it won't give us the image sources,
    // so... we have to scrape their internal json

    // iterate through every script until we find something that matches our regex
    let internal_json_regex =
        regex::Regex::new(r#"(?:\(function\(\)\{google\.jl=\{.+?)var \w=(\{".+?\});"#)?;
    let mut internal_json = None;
    let dom = scraper::Html::parse_document(body);
    for script in dom.select(&Selector::parse("script").unwrap()) {
        let script = script.inner_html();
        if let Some(captures) = internal_json_regex.captures(&script).and_then(|c| c.get(1)) {
            internal_json = Some(captures.as_str().to_string());
            break;
        }
    }

    let internal_json =
        internal_json.ok_or_else(|| eyre!("couldn't get internal json for google images"))?;
    let internal_json: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&internal_json)?;

    let mut image_results = Vec::new();
    for element_json in internal_json.values() {
        // the internal json uses arrays instead of maps, which makes it kinda hard to
        // use and also probably pretty unstable

        let Some(element_json) = element_json
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
        else {
            continue;
        };

        let Some((image_url, width, height)) = element_json
            .get(3)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
        else {
            warn!("couldn't get image data from google images json");
            continue;
        };

        // this is probably pretty brittle, hopefully google doesn't break it any time
        // soon
        let Some(page) = element_json
            .get(9)
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("2003"))
            .and_then(|v| v.as_array())
        else {
            warn!("couldn't get page data from google images json");
            continue;
        };
        let Some(page_url) = page.get(2).and_then(|v| v.as_str()).map(|s| s.to_string()) else {
            warn!("couldn't get page url from google images json");
            continue;
        };
        let Some(title) = page.get(3).and_then(|v| v.as_str()).map(|s| s.to_string()) else {
            warn!("couldn't get page title from google images json");
            continue;
        };

        image_results.push(EngineImageResult {
            image_url,
            page_url,
            title,
            width,
            height,
        });
    }

    Ok(EngineImagesResponse { image_results })
}
