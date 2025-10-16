use base64::Engine;
use eyre::eyre;
use rand::Rng;
use scraper::{ElementRef, Html, Selector};
use tracing::warn;
use url::Url;

use crate::{
    engines::{EngineImageResult, EngineImagesResponse, EngineResponse, CLIENT},
    parse::{parse_html_response_with_opts, ParseOpts, QueryMethod},
};

pub fn request(query: &str) -> reqwest::RequestBuilder {
    let cvid = generate_cvid();
    let url = Url::parse_with_params(
        "https://www.bing.com/search",
        &[
            ("q", query),
            ("pq", query),
            ("cvid", &cvid),
            ("filters", "rcrse:\"1\""), // filters=rcrse:"1" makes it not try to autocorrect
            ("FORM", "PERE"),
            ("ghc", "1"),
            ("lq", "0"),
            ("qs", "n"),
            ("sk", ""),
            ("sp", "-1"),
        ],
    )
    .unwrap();
    CLIENT
        .get(url)
        .header("Cookie", &format!("SRCHHPGUSR=IG={}", cvid))
}

fn generate_cvid() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    parse_html_response_with_opts(
        body,
        ParseOpts::new()
            .result("#b_results > li.b_algo")
            .title(".b_algo h2 > a")
            .href(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let url = el
                    .select(&Selector::parse("a[href]").unwrap())
                    .next()
                    .and_then(|n| n.value().attr("href"))
                    .unwrap_or_default();
                clean_url(url)
            })))
            .description(QueryMethod::Manual(Box::new(|el: &ElementRef| {
                let mut description = String::new();
                for inner_node in el
                    .select(
                        &Selector::parse(".b_caption > p, p.b_algoSlug, .b_caption .ipText")
                            .unwrap(),
                    )
                    .next()
                    .map(|n| n.children().collect::<Vec<_>>())
                    .unwrap_or_default()
                {
                    match inner_node.value() {
                        scraper::Node::Text(t) => {
                            description.push_str(&t.text);
                        }
                        scraper::Node::Element(inner_el) => {
                            if !inner_el
                                .has_class("algoSlug_icon", scraper::CaseSensitivity::CaseSensitive)
                            {
                                let element_ref = ElementRef::wrap(inner_node).unwrap();
                                description.push_str(&element_ref.text().collect::<String>());
                            }
                        }
                        _ => {}
                    }
                }

                Ok(description)
            }))),
    )
}

pub fn request_images(query: &str) -> reqwest::RequestBuilder {
    CLIENT.get(
        Url::parse_with_params(
            "https://www.bing.com/images/async",
            &[
                ("q", query),
                ("async", "content"),
                ("first", "1"),
                ("count", "35"),
            ],
        )
        .unwrap(),
    )
}

#[tracing::instrument(skip(body))]
pub fn parse_images_response(body: &str) -> eyre::Result<EngineImagesResponse> {
    let dom = Html::parse_document(body);

    let mut image_results = Vec::new();

    let image_container_el_sel = Selector::parse(".imgpt").unwrap();
    let image_el_sel = Selector::parse(".iusc").unwrap();
    for image_container_el in dom.select(&image_container_el_sel) {
        let image_el = image_container_el
            .select(&image_el_sel)
            .next()
            .ok_or_else(|| eyre!("no image element found"))?;

        // parse the "m" attribute as json
        let Some(data) = image_el.value().attr("m") else {
            // this is normal, i think
            continue;
        };
        let data = serde_json::from_str::<serde_json::Value>(data)?;
        let page_url = data
            .get("purl")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let image_url = data
            // short for media url, probably
            .get("murl")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let page_title = data
            .get("t")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            // bing adds these unicode characters around matches
            .replace(['', ''], "");

        // the text looks like "1200 x 1600 · jpegWikipedia"
        // (the last part is incorrectly parsed since the actual text is inside another
        // element but this is already good enough for our purposes)
        let text = image_container_el.text().collect::<String>();
        let width_height: Vec<u64> = text
            .split(" · ")
            .next()
            .unwrap_or_default()
            .split(" x ")
            .map(|s| s.parse().unwrap_or_default())
            .collect();
        let (width, height) = match width_height.as_slice() {
            [width, height] => (*width, *height),
            _ => {
                warn!("couldn't get width and height from text \"{text}\"");
                continue;
            }
        };

        image_results.push(EngineImageResult {
            page_url: page_url.to_string(),
            image_url: image_url.to_string(),
            title: page_title.to_string(),
            width,
            height,
        });
    }

    Ok(EngineImagesResponse { image_results })
}

fn clean_url(url: &str) -> eyre::Result<String> {
    // clean up bing's tracking urls
    if url.starts_with("https://www.bing.com/ck/a?") {
        // get the u param
        let url = Url::parse(url)?;
        let u = url
            .query_pairs()
            .find(|(key, _)| key == "u")
            .unwrap_or_default()
            .1;
        // cut off the "a1" and base64 decode
        let u = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&u[2..])
            .unwrap_or_default();
        // convert to utf8
        Ok(String::from_utf8_lossy(&u).to_string())
    } else {
        Ok(url.to_string())
    }
}
