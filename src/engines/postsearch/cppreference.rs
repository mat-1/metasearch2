use maud::{html, PreEscaped};
use scraper::{Html, Selector};

use crate::engines::{HttpResponse, Response, CLIENT};

pub async fn request(response: &Response) -> Option<wreq::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result.result.url.contains("cppreference.com") {
            return Some(CLIENT.get(search_result.result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(HttpResponse { res, body, .. }: &HttpResponse) -> Option<PreEscaped<String>> {
    let url = res.url().clone();
    let dom = Html::parse_document(body);

    //extract heading
    let page_title = dom
        .select(&Selector::parse("#firstHeading").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    //extract declaration
    let item_decl = dom
        .select(&Selector::parse(".t-dcl-begin").unwrap())
        .next()
        .map(|el| el.html())
        .unwrap_or_default();

    //extract first paragraph of text
    let main_content_selector = Selector::parse("#mw-content-text > .mw-parser-output > p").unwrap();
    let doc_html = dom
        .select(&main_content_selector)
        .next()
        .map(|doc| doc.html())
        .unwrap_or_default();

    let example_html = dom
        .select(&Selector::parse(".t-example pre").unwrap())
        .next()
        .map(|el| el.html())
        .unwrap_or_default();

        
    if item_decl.is_empty() && doc_html.is_empty() && example_html.is_empty() {
        return None;
    }


    // sanitize the example by itself so it can be separated in the postanswer
    let sanitized_example = if !example_html.is_empty() {
        ammonia::Builder::default()
            .link_rel(None)
            .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
            .clean(&example_html)
            .to_string()
    } else {
        String::new()
    };



    let sanitized_html = ammonia::Builder::default()
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
        .clean(&format!("{item_decl}{doc_html}"))
        .to_string();

    Some(html! {
        h2 {
            "C++ Reference "
            a href=(url) { (page_title) }
        }
        div.infobox-cppreference-doc {
            //render the basic info
            (PreEscaped(sanitized_html))

            //render the example only if it was found
            @if !sanitized_example.is_empty() {
            
                hr
                
                div.infobox-cppreference-example {
                    h3 { "Example" }
                    (PreEscaped(sanitized_example))
                }
            }
        }
    })

}
