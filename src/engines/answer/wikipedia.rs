use std::collections::HashMap;

use reqwest::Url;
use serde::Deserialize;

use crate::engines::{EngineResponse, CLIENT};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    CLIENT
        .get(
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
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
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

    let extract = extract.replace("( )", "").replace("()", "");

    let page_title = title.replace(' ', "_");
    let page_url = format!("https://en.wikipedia.org/wiki/{page_title}");

    Ok(EngineResponse::infobox_html(format!(
        r#"<a href="{page_url}"><h2>{title}</h2></a><p>{extract}</p>"#,
        page_url = html_escape::encode_quoted_attribute(&page_url),
        title = html_escape::encode_text(title),
        extract = html_escape::encode_text(&extract),
    )))
}
