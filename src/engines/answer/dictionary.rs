use std::collections::HashMap;

use eyre::eyre;
use serde::Deserialize;
use url::Url;

use crate::engines::{EngineResponse, HttpResponse, RequestResponse, CLIENT};

use super::regex;

pub fn request(query: &str) -> RequestResponse {
    // if the query starts with "define " then use that, otherwise abort
    let re = regex!(r"^define\s+(\w+)$");
    let query = match re.captures(query) {
        Some(caps) => caps.get(1).unwrap().as_str(),
        None => return RequestResponse::None,
    }
    .to_lowercase();

    CLIENT
        .get(
            Url::parse(
                format!(
                    "https://en.wiktionary.org/api/rest_v1/page/definition/{}",
                    urlencoding::encode(&query)
                )
                .as_str(),
            )
            .unwrap(),
        )
        .into()
}

#[derive(Debug, Deserialize)]
pub struct WiktionaryResponse(pub HashMap<String, Vec<WiktionaryEntry>>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiktionaryEntry {
    pub part_of_speech: String,
    pub language: String,
    pub definitions: Vec<WiktionaryDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiktionaryDefinition {
    pub definition: String,
    #[serde(default)]
    pub examples: Vec<String>,
}

pub fn parse_response(HttpResponse { res, body }: &HttpResponse) -> eyre::Result<EngineResponse> {
    let url = res.url();

    let Ok(res) = serde_json::from_str::<WiktionaryResponse>(body) else {
        return Ok(EngineResponse::new());
    };

    let mediawiki_key = url
        .path_segments()
        .ok_or_else(|| eyre!("url has no path segments"))?
        .last()
        .ok_or_else(|| eyre!("url has no last path segment"))?;

    let word = key_to_title(mediawiki_key);

    let mut html = String::new();

    let Some(entries) = res.0.get("en") else {
        return Ok(EngineResponse::new());
    };

    html.push_str(&format!(
        "<h2 class=\"answer-dictionary-word\"><a href=\"https://en.wiktionary.org/wiki/{mediawiki_key}\">{word}</a></h2>",
        mediawiki_key = html_escape::encode_safe(mediawiki_key),
        word = html_escape::encode_safe(&word),
    ));

    let mut cleaner = ammonia::Builder::default();
    cleaner
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            Url::parse("https://en.wiktionary.org").unwrap(),
        ));

    for entry in entries {
        html.push_str(&format!(
            "<span class=\"answer-dictionary-part-of-speech\">{part_of_speech}</span>",
            part_of_speech = html_escape::encode_safe(&entry.part_of_speech.to_lowercase())
        ));

        html.push_str("<ol>");
        let mut previous_definitions = Vec::<String>::new();
        for definition in &entry.definitions {
            if definition.definition.is_empty() {
                // wiktionary does this sometimes, for example https://en.wiktionary.org/api/rest_v1/page/definition/variance
                continue;
            }
            if previous_definitions
                .iter()
                .any(|d| d.contains(&definition.definition))
            {
                // wiktionary will sometimes duplicate definitions, for example https://en.wiktionary.org/api/rest_v1/page/definition/google
                continue;
            }
            previous_definitions.push(definition.definition.clone());

            html.push_str("<li class=\"answer-dictionary-definition\">");
            let definition_html = cleaner
                .clean(&definition.definition.replace('“', "\""))
                .to_string();

            html.push_str(&format!("<p>{definition_html}</p>"));

            if !definition.examples.is_empty() {
                for example in &definition.examples {
                    let example_html = cleaner.clean(example).to_string();
                    html.push_str(&format!("<blockquote class=\"answer-dictionary-example\">{example_html}</blockquote>"));
                }
            }
            html.push_str("</li>");
        }
        html.push_str("</ol>");
    }

    Ok(EngineResponse::answer_html(html))
}

fn key_to_title(key: &str) -> String {
    // https://github.com/wikimedia/mediawiki-title
    // In general, the page title is converted to the mediawiki DB key format by
    // trimming spaces, replacing whitespace symbols to underscores and applying
    // wiki-specific capitalization rules.

    let title = key.trim().replace('_', " ");
    let mut c = title.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
