use maud::{html, PreEscaped};
use scraper::{Html, Selector};
use serde::Deserialize;
use tracing::error;

use crate::engines::{Engine, HttpResponse, Response, CLIENT};

#[derive(Deserialize)]
pub struct MdnConfig {
    pub max_sections: usize,
}

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result
            .result
            .url
            .starts_with("https://developer.mozilla.org/en-US/docs/Web")
        {
            return Some(CLIENT.get(search_result.result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(
    HttpResponse { res, body, config }: &HttpResponse,
) -> Option<PreEscaped<String>> {
    let config_toml = config.engines.get(Engine::Mdn).extra.clone();
    let config: MdnConfig = match toml::Value::Table(config_toml).try_into() {
        Ok(args) => args,
        Err(err) => {
            error!("Failed to parse Mdn config: {err}");
            return None;
        }
    };

    let url = res.url().clone();

    let dom = Html::parse_document(body);

    let page_title = dom
        .select(&Selector::parse("header > h1").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let doc_query = Selector::parse(".section-content").unwrap();

    let max_sections = if config.max_sections == 0 {
        usize::MAX
    } else {
        config.max_sections
    };

    let doc_html = dom
        .select(&doc_query)
        .map(|doc| doc.inner_html())
        .take(max_sections)
        .collect::<Vec<_>>()
        .join("<br>");

    let doc_html = ammonia::Builder::default()
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
        .clean(&doc_html)
        .to_string();

    Some(html! {
        h2 {
            a href=(url) { (page_title) }
        }
        div.infobox-mdn-article {
            (PreEscaped(doc_html))
        }
    })
}
