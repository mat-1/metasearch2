use scraper::{ElementRef, Html, Selector};

use crate::engines::{HttpResponse, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result.url.starts_with("https://minecraft.wiki/w/") {
            return Some(CLIENT.get(search_result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(HttpResponse { res, body, .. }: &HttpResponse) -> Option<String> {
    let url = res.url().clone();

    let dom = Html::parse_document(body);

    let page_title = dom
        .select(&Selector::parse("#firstHeading").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let doc_query = Selector::parse(".mw-parser-output > p").unwrap();

    let doc_html = dom
        .select(&doc_query)
        .next()
        .map(|doc| doc.html())
        .unwrap_or_default();

    let doc_html = ammonia::Builder::default()
        .link_rel(None)
        .add_allowed_classes("div", ["notaninfobox", "mcw-mainpage-icon"])
        .add_allowed_classes("pre", ["noexcerpt", "navigation-not-searchable"])
        .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
        .clean(&doc_html)
        .to_string();

    let title_html = format!(
        r#"<h2><a href="{url}">{title}</a></h2>"#,
        url = html_escape::encode_quoted_attribute(&url.to_string()),
        title = html_escape::encode_safe(&page_title),
    );

    Some(format!(
        r#"{title_html}<div class="infobox-minecraft_wiki-article">{doc_html}</div>"#
    ))
}
