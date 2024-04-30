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
    html! {
        div.image-result {
            a.image-result-anchor rel="noreferrer" href=(result.result.image_url) target="_blank" {
                img loading="lazy" src=(result.result.image_url);
            }
            a.image-result-page-anchor href=(result.result.page_url) {
                span.image-result-page-url.search-result-url { (result.result.page_url) }
                span.image-result-title { (result.result.title) }
            }
        }
    }
}
