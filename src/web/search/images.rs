use maud::{html, PreEscaped};

use crate::{
    config::Config,
    engines::{self, EngineImageResult, ImagesResponse},
    web::search::render_engine_list,
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
    let image_src = if config.image_search.proxy.enabled {
        // serialize url params
        let escaped_param =
            url::form_urlencoded::byte_serialize(original_image_src.as_bytes()).collect::<String>();
        format!("/image-proxy?url={escaped_param}")
    } else {
        original_image_src.to_string()
    };
    html! {
        div.image-result {
            a.image-result-anchor rel="noreferrer" href=(original_image_src) target="_blank" {
                div.image-result-img-container {
                    img loading="lazy" src=(image_src) width=(result.result.width) height=(result.result.height);
                }
            }
            a.image-result-page-anchor href=(result.result.page_url) {
                span.image-result-page-url.search-result-url { (result.result.page_url) }
                span.image-result-title { (result.result.title) }
            }
            @if config.image_search.show_engines {
                {(render_engine_list(&result.engines.iter().copied().collect::<Vec<_>>(), &config))}
            }
        }
    }
}
