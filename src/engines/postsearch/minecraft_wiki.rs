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

pub fn parse_response(
    HttpResponse {
        res,
        body,
        config: _config,
    }: &HttpResponse,
) -> Option<String> {
    let url = res.url().clone();

    let dom = Html::parse_document(body);

    let page_title = dom
        .select(&Selector::parse("#firstHeading").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let doc_query = Selector::parse(".mw-parser-output").unwrap(); // > :not(h2:has(>#Gallery) ~ *)").unwrap();

    let doc_html = dom
        .select(&doc_query)
        .next()
        .map(|doc| strip_gallery(doc))
        .unwrap_or_default()
        .join("");

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

fn strip_gallery(doc: ElementRef) -> Vec<String> {
    let mut gallery = false;
    doc.children()
        .filter(|elem| {
            let value = elem.value();
            if gallery {
                return false;
            }
            match value {
                scraper::Node::Element(_) => {
                    let elem = ElementRef::wrap(elem.clone()).unwrap();
                    let is_gallery_title = elem.first_child().map_or(false, |elem| {
                        elem.value().as_element().map_or(false, |_| {
                            let elem = ElementRef::wrap(elem).unwrap();
                            elem.text().collect::<String>() == "Gallery"
                        })
                    });
                    if is_gallery_title {
                        gallery = true;
                        return false;
                    }
                    true
                }
                _ => true,
            }
        })
        .map(|elem| {
            ElementRef::wrap(elem)
                .map(|elem| elem.html())
                .unwrap_or_default()
        })
        .collect()
}
