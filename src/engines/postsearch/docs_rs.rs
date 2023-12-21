use reqwest::Url;
use scraper::{Html, Selector};

use crate::engines::{Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result.url.starts_with("https://docs.rs/") {
            return Some(CLIENT.get(search_result.url.as_str()).header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
            ));
        }
    }

    None
}

pub fn parse_response(body: &str) -> Option<String> {
    let dom = Html::parse_document(body);

    let title = dom
        .select(&Selector::parse("h2 a").unwrap())
        .next()?
        .text()
        .collect::<String>();
    let version = dom
        .select(&Selector::parse("h2 .version").unwrap())
        .next()?
        .text()
        .collect::<String>();

    let url = Url::join(
        &Url::parse("https://docs.rs").unwrap(),
        &dom.select(
            &Selector::parse("ul.pure-menu-list li.pure-menu-item:nth-last-child(2) a").unwrap(),
        )
        .next()?
        .value()
        .attr("href")?
        .replace("/crate/", "/"),
    )
    .ok()?;

    let doc_query = Selector::parse(".docblock").unwrap();

    let doc = dom.select(&doc_query).next()?;
    let doc_html = doc.inner_html();
    let doc_html = ammonia::Builder::default()
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            Url::parse("https://docs.rs").unwrap(),
        ))
        .clean(&doc_html)
        .to_string();

    Some(format!(
        r#"<h2>Crate <a href="{url}">{title} {version}</a></h2>
<div class="infobox-docs.rs-answer">{doc_html}</div>"#,
        url = html_escape::encode_quoted_attribute(&url.to_string()),
        title = html_escape::encode_text(&title),
    ))
}
