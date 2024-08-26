use std::collections::HashMap;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use tracing::error;

use crate::{config::Config, engines};

pub async fn route(
    Query(params): Query<HashMap<String, String>>,
    Extension(config): Extension<Config>,
) -> Response {
    let image_search_config = &config.image_search;
    let proxy_config = &image_search_config.proxy;
    if !image_search_config.enabled || !proxy_config.enabled {
        return (StatusCode::FORBIDDEN, "Image proxy is disabled").into_response();
    };
    let url = params.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing `url` parameter").into_response();
    }

    let mut res = match engines::CLIENT
        .get(&url)
        .header("accept", "image/*")
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            error!("Image proxy error for {url}: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Image proxy error").into_response();
        }
    };

    let max_size = proxy_config.max_download_size;

    if res.content_length().unwrap_or_default() > max_size {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Image too large").into_response();
    }
    // validate content-type
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    if !content_type.starts_with("image/") {
        return (StatusCode::BAD_REQUEST, "Not an image").into_response();
    }

    let mut image_bytes = Vec::new();
    while let Ok(Some(chunk)) = res.chunk().await {
        image_bytes.extend_from_slice(&chunk);
        if image_bytes.len() as u64 > max_size {
            return (StatusCode::PAYLOAD_TOO_LARGE, "Image too large").into_response();
        }
    }

    (
        [
            (axum::http::header::CONTENT_TYPE, content_type),
            (
                axum::http::header::CACHE_CONTROL,
                "public, max-age=31536000".to_owned(),
            ),
        ],
        image_bytes,
    )
        .into_response()
}
