use scraper::{Html, Selector};
use url::Url;

use crate::engines::{answer::regex, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if regex!(r"^https:\/\/github\.com\/[\w-]+\/[\w.-]+$").is_match(&search_result.url) {
            return Some(CLIENT.get(search_result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(body: &str) -> Option<String> {
    let dom = Html::parse_document(body);

    let url_relative = dom
        .select(
            &Selector::parse("main #repository-container-header strong[itemprop='name'] > a")
                .unwrap(),
        )
        .next()?
        .value()
        .attr("href")?;
    let url = format!("https://github.com{url_relative}");

    let embedded_data_script = dom
        .select(&Selector::parse("script[data-target='react-partial.embeddedData']").unwrap())
        .last()?
        .inner_html();
    let embedded_data = serde_json::from_str::<serde_json::Value>(&embedded_data_script).ok()?;
    let readme_html = embedded_data
        .get("props")?
        .get("initialPayload")?
        .get("overview")?
        .get("overviewFiles")?
        .as_array()?
        .first()?
        .get("richText")?
        .as_str()?;

    let mut readme_html = ammonia::Builder::default()
        .link_rel(None)
        .add_allowed_classes("div", &["markdown-alert"])
        .add_allowed_classes("p", &["markdown-alert-title"])
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            Url::parse("https://github.com").unwrap(),
        ))
        .clean(&readme_html)
        .to_string();

    let readme_dom = Html::parse_fragment(&readme_html);
    let title = if let Some(title_el) = readme_dom.select(&Selector::parse("h1").unwrap()).next() {
        let title_html = title_el.html().trim().to_string();
        if readme_html.starts_with(&title_html) {
            readme_html = readme_html[title_html.len()..].to_string();
        }
        title_el.text().collect::<String>()
    } else {
        dom.select(
            &Selector::parse("main #repository-container-header strong[itemprop='name'] > a")
                .unwrap(),
        )
        .next()?
        .text()
        .collect::<String>()
    };

    Some(format!(
        r#"<a href="{url}"><h1>{title}</h1></a>
<div class="infobox-github-readme">{readme_html}</div>"#,
        url = html_escape::encode_quoted_attribute(&url),
        title = html_escape::encode_text(&title),
    ))
}
