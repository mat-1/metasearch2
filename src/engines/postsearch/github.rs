use scraper::{Html, Selector};
use url::Url;

use crate::engines::{Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result.url.starts_with("https://github.com/") {
            return Some(CLIENT.get(search_result.url.as_str()).header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
            ));
        }
    }

    None
}

pub fn parse_response(body: &str, _url: Url) -> Option<String> {
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

    let readme = dom.select(&Selector::parse("article").unwrap()).next()?;
    let readme_html = readme.inner_html().trim().to_string();

    let mut readme_html = ammonia::Builder::default()
        .link_rel(None)
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
