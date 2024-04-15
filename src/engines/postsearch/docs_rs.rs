use scraper::{Html, Selector};

use crate::engines::{HttpResponse, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result.url.starts_with("https://docs.rs/") {
            return Some(CLIENT.get(search_result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(
    HttpResponse {
        res,
        body,
        config: _config,
    }: &HttpResponse,
) -> Option<String> {
    let url = res.url().clone();

    let dom = Html::parse_document(body);

    let version = dom
        .select(&Selector::parse("h2 .version").unwrap())
        .next()?
        .text()
        .collect::<String>();

    let page_title = dom
        .select(&Selector::parse("h1").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let doc_query = Selector::parse(".docblock").unwrap();

    let doc_html = dom
        .select(&doc_query)
        .next()
        .map(|doc| doc.inner_html())
        .unwrap_or_default();

    let item_decl = dom
        .select(&Selector::parse(".item-decl").unwrap())
        .next()
        .map(|el| el.html())
        .unwrap_or_default();

    let doc_html = ammonia::Builder::default()
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
        .clean(&format!("{item_decl}{doc_html}"))
        .to_string();

    let (category, title) = page_title.split_once(' ').unwrap_or(("", &page_title));

    let title_html = if category == "Crate" {
        format!(
            r#"<h2>{category} <a href="{url}">{title}</a> <span class="infobox-docs_rs-version">{version}</span></h2>"#,
            url = html_escape::encode_quoted_attribute(&url.to_string()),
            title = html_escape::encode_safe(&title),
            version = html_escape::encode_safe(&version),
        )
    } else {
        format!(
            r#"<h2>{category} <a href="{url}">{title}</a></h2>"#,
            url = html_escape::encode_quoted_attribute(&url.to_string()),
            title = html_escape::encode_safe(&title),
        )
    };

    Some(format!(
        r#"{title_html}<div class="infobox-docs.rs-doc">{doc_html}</div>"#
    ))
}
