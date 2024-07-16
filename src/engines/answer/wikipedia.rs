use std::collections::HashMap;

use maud::html;
use serde::Deserialize;
use url::Url;

use crate::engines::{EngineResponse, RequestResponse, CLIENT};

use super::colorpicker;

pub fn request(mut query: &str) -> RequestResponse {
    if !colorpicker::MatchedColorModel::new(query).is_empty() {
        // "color picker" is a wikipedia article but we only want to show the
        // actual color picker answer
        return RequestResponse::None;
    }

    // adding "wikipedia" to the start or end of your query is common when you
    // want to get a wikipedia article
    if let Some(stripped_query) = query.strip_suffix(" wikipedia") {
        query = stripped_query
    } else if let Some(stripped_query) = query.strip_prefix("wikipedia ") {
        query = stripped_query
    }

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
        .into()
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

    let mut previous_extract = String::new();
    let mut extract = extract.clone();
    while previous_extract != extract {
        previous_extract.clone_from(&extract);
        extract = extract
            .replace("( ", "(")
            .replace("(, ", "(")
            .replace("(; ", "(")
            .replace(" ()", "")
            .replace("()", "");
    }

    let page_title = title.replace(' ', "_");
    let page_url = format!("https://en.wikipedia.org/wiki/{page_title}");

    Ok(EngineResponse::infobox_html(html! {
        a href=(page_url) {
            h2 { (title) }
        }
        p { (extract) }
    }))
}
