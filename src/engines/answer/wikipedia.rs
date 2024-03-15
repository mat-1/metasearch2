use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::engines::{EngineResponse, CLIENT};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://en.wikipedia.org/w/api.php",
            &[
                ("format", "json"),
                ("action", "query"),
                ("prop", "extracts|pageimages"),
                ("exintro", ""),
                ("explaintext", ""),
                ("redirects", "1"),
                ("exsentences", "2"),
                ("titles", query),
            ],
        )
        .unwrap(),
    )
}

#[derive(Debug, Deserialize)]
pub struct WikipediaResponse {
    pub batchcomplete: String,
    pub query: WikipediaQuery,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaQuery {
    pub pages: HashMap<String, WikipediaPage>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaPage {
    pub pageid: u64,
    pub ns: u64,
    pub title: String,
    pub extract: String,
    pub thumbnail: Option<WikipediaThumbnail>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaThumbnail {
    pub source: String,
    pub width: u64,
    pub height: u64,
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let Ok(res) = serde_json::from_str::<WikipediaResponse>(body) else {
        return Ok(EngineResponse::new());
    };

    let pages: Vec<(String, WikipediaPage)> = res.query.pages.into_iter().collect();

    if pages.is_empty() || pages[0].0 == "-1" {
        return Ok(EngineResponse::new());
    }

    let page = &pages[0].1;
    let WikipediaPage {
        pageid: _,
        ns: _,
        title,
        extract,
        thumbnail: _,
    } = page;
    if extract.ends_with(':') {
        return Ok(EngineResponse::new());
    }

    let mut previous_extract = "".to_string();
    let mut extract = extract.clone();
    while previous_extract != extract {
        previous_extract = extract.clone();
        extract = extract
            .replace("( ", "(")
            .replace("(, ", "(")
            .replace("(; ", "(")
            .replace("()", "");
    }

    let page_title = title.replace(' ', "_");
    let page_url = format!("https://en.wikipedia.org/wiki/{page_title}");

    Ok(EngineResponse::infobox_html(format!(
        r#"<a href="{page_url}"><h2>{title}</h2></a><p>{extract}</p>"#,
        page_url = html_escape::encode_quoted_attribute(&page_url),
        title = html_escape::encode_text(title),
        extract = html_escape::encode_text(&extract),
    )))
}
