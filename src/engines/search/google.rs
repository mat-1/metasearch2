use std::{
    sync::{Arc, LazyLock},
    time::Instant,
};

use eyre::eyre;
use parking_lot::RwLock;
use rand::distr::{slice::Choose, SampleString};
use scraper::{ElementRef, Selector};
use tracing::warn;
use url::Url;

use crate::{
    engines::{EngineImageResult, EngineImagesResponse, EngineResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts, QueryMethod},
};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    let url = Url::parse_with_params(
        "https://www.google.com/search",
        &[
            ("q", query),
            // nfpr makes it not try to autocorrect
            ("nfpr", "1"),
            ("filter", "0"),
            ("start", "0"),
            // mobile search, lets us easily search without js
            ("asearch", "arc"),
            // required for mobile search to work
            ("async", &generate_async_value()),
        ],
    )
    .unwrap();
    CLIENT.get(url)
}

fn generate_async_value() -> String {
    // https://github.com/searxng/searxng/blob/08a90d46d6f23607ddecf2a2d9fa216df69d2fac/searx/engines/google.py#L80

    let use_ac = "use_ac:true";
    let fmt = "_fmt:prog";

    static CURRENT_RANDOM_CHARACTERS: LazyLock<Arc<RwLock<(String, Instant)>>> =
        LazyLock::new(|| Arc::new(RwLock::new((generate_new_arc_id_random(), Instant::now()))));
    let (random_characters, last_set) = CURRENT_RANDOM_CHARACTERS.read().clone();

    if last_set.elapsed().as_secs() > 60 * 60 {
        // copy what searxng does and rotate every hour
        let mut arc_id = CURRENT_RANDOM_CHARACTERS.write();
        *arc_id = (generate_new_arc_id_random(), Instant::now());
    }

    let page_number = 1;
    let arc_id = format!(
        "arc_id:srp_{random_characters}_{skip}",
        skip = 100 + page_number * 10
    );

    format!("{arc_id},{use_ac},{fmt}")
}

fn generate_new_arc_id_random() -> String {
    let candidate_characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";

    Choose::new(&candidate_characters.chars().collect::<Vec<_>>())
        .unwrap()
        .sample_string(&mut rand::rng(), 23)
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            // xpd is weird, some results have it but it's usually used for ads?
            // the :first-child filters out the ads though since for ads the first child is always a
            // span
            .result("[jscontroller=SC7lYd]")
            .title("h3")
            .href("a[href]")
            .description(
                "div[data-sncf='2'], div[data-sncf='1,2'], div[style='-webkit-line-clamp:2']",
            )
            .featured_snippet("block-component")
            .featured_snippet_description(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let mut description = String::new();

                // role="heading"
                if let Some(heading_el) = el
                    .select(&Selector::parse("div[role='heading']").unwrap())
                    .next()
                {
                    description.push_str(&format!("{}\n\n", heading_el.text().collect::<String>()));
                }

                if let Some(description_container_el) = el
                    .select(&Selector::parse("div[data-attrid='wa:/description'] > span:first-child").unwrap())
                    .next()
                {
                    description.push_str(&iter_featured_snippet_children(&description_container_el));
                }
                else if let Some(description_list_el) = el
                    .select(&Selector::parse("ul").unwrap())
                    .next()
                {
                    // render as bullet points
                    for li in description_list_el.select(&Selector::parse("li").unwrap()) {
                        let text = li.text().collect::<String>();
                        description.push_str(&format!("• {text}\n"));
                    }
                }

                Ok(description)
            })))
            .featured_snippet_title(".g > div[lang] a h3, div[lang] > div[style='position:relative'] a h3")
            .featured_snippet_href(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let url = el
                    .select(&Selector::parse(".g > div[lang] a:has(h3), div[lang] > div[style='position:relative'] a:has(h3)").unwrap())
                    .next()
                    .and_then(|n| n.value().attr("href"))
                    .unwrap_or_default();
                clean_url(url)
            }))),
    )
}

// Google autocomplete responses sometimes include clickable links that include
// text that we shouldn't show.
// We can filter for these by removing any elements matching
// [data-ved]:not([data-send-open-event])
fn iter_featured_snippet_children(el: &ElementRef) -> String {
    let mut description = String::new();
    recursive_iter_featured_snippet_children(&mut description, el);
    description
}
fn recursive_iter_featured_snippet_children(description: &mut String, el: &ElementRef) {
    for inner_node in el.children() {
        match inner_node.value() {
            scraper::Node::Text(t) => {
                description.push_str(&t.text);
            }
            scraper::Node::Element(inner_el) => {
                if inner_el.attr("data-ved").is_none()
                    || inner_el.attr("data-send-open-event").is_some()
                {
                    recursive_iter_featured_snippet_children(
                        description,
                        &ElementRef::wrap(inner_node).unwrap(),
                    );
                }
            }
            _ => {}
        }
    }
}

pub fn request_autocomplete(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://suggestqueries.google.com/complete/search",
            &[
                ("output", "firefox"),
                ("client", "firefox"),
                ("hl", "US-en"),
                ("q", query),
            ],
        )
        .unwrap(),
    )
}

pub fn parse_autocomplete_response(body: &str) -> eyre::Result<Vec<String>> {
    let res = serde_json::from_str::<Vec<serde_json::Value>>(body)?;
    Ok(res
        .into_iter()
        .nth(1)
        .unwrap_or_default()
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect())
}

pub fn request_images(query: &str) -> reqwest::RequestBuilder {
    // ok so google also has a json api for images BUT it gives us less results
    CLIENT.get(
        Url::parse_with_params(
            "https://www.google.com/search",
            &[("q", query), ("udm", "2"), ("prmd", "ivsnmbtz")],
        )
        .unwrap(),
    )
}

pub fn parse_images_response(body: &str) -> eyre::Result<EngineImagesResponse> {
    // we can't just scrape the html because it won't give us the image sources,
    // so... we have to scrape their internal json

    // iterate through every script until we find something that matches our regex
    let internal_json_regex =
        regex::Regex::new(r#"(?:\(function\(\)\{google\.jl=\{.+?)var \w=(\{".+?\});"#)?;
    let mut internal_json = None;
    let dom = scraper::Html::parse_document(body);
    for script in dom.select(&Selector::parse("script").unwrap()) {
        let script = script.inner_html();
        if let Some(captures) = internal_json_regex.captures(&script).and_then(|c| c.get(1)) {
            internal_json = Some(captures.as_str().to_string());
            break;
        }
    }

    let internal_json =
        internal_json.ok_or_else(|| eyre!("couldn't get internal json for google images"))?;
    let internal_json: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&internal_json)?;

    let mut image_results = Vec::new();
    for element_json in internal_json.values() {
        // the internal json uses arrays instead of maps, which makes it kinda hard to
        // use and also probably pretty unstable

        let Some(element_json) = element_json
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
        else {
            continue;
        };

        let Some((image_url, width, height)) = element_json
            .get(3)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
        else {
            warn!("couldn't get image data from google images json");
            continue;
        };

        // this is probably pretty brittle, hopefully google doesn't break it any time
        // soon
        let Some(page) = element_json
            .get(9)
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("2003"))
            .and_then(|v| v.as_array())
        else {
            warn!("couldn't get page data from google images json");
            continue;
        };
        let Some(page_url) = page.get(2).and_then(|v| v.as_str()).map(|s| s.to_string()) else {
            warn!("couldn't get page url from google images json");
            continue;
        };
        let Some(title) = page.get(3).and_then(|v| v.as_str()).map(|s| s.to_string()) else {
            warn!("couldn't get page title from google images json");
            continue;
        };

        image_results.push(EngineImageResult {
            image_url,
            page_url,
            title,
            width,
            height,
        });
    }

    Ok(EngineImagesResponse { image_results })
}

fn clean_url(url: &str) -> eyre::Result<String> {
    if url.starts_with("/url?q=") {
        // get the q param
        let url = Url::parse(format!("https://www.google.com{url}").as_str())?;
        let q = url
            .query_pairs()
            .find(|(key, _)| key == "q")
            .unwrap_or_default()
            .1;
        Ok(q.to_string())
    } else {
        Ok(url.to_string())
    }
}
