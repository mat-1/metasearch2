use maud::{html, PreEscaped};

use crate::{
    config::Config,
    engines::{self, EngineImageResult, ImagesResponse},
};

pub fn render_results(response: ImagesResponse) -> PreEscaped<String> {
    html! {
        div.image-results {
            @for image in &response.image_results {
                (render_image_result(image, &response.config))
            }
        }
    }
}

fn render_image_result(
    result: &engines::SearchResult<EngineImageResult>,
    config: &Config,
) -> PreEscaped<String> {
    let original_image_src = &result.result.image_url;
    let image_src = if config.image_search.proxy.enabled.unwrap() {
        // serialize url params
        let escaped_param =
            url::form_urlencoded::byte_serialize(original_image_src.as_bytes()).collect::<String>();
        format!("/image-proxy?url={}", escaped_param)
    } else {
        original_image_src.to_string()
    };
    html! {
        div.image-result {
            a.image-result-anchor rel="noreferrer" href=(original_image_src) target="_blank" {
                img loading="lazy" src=(image_src);
            }
            a.image-result-page-anchor href=(result.result.page_url) {
                span.image-result-page-url.search-result-url { (result.result.page_url) }
                span.image-result-title { (result.result.title) }
            }
        }
    }
}
