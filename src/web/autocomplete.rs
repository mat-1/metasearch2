use std::collections::HashMap;

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use tracing::error;

use crate::{config::Config, engines};

pub async fn route(
    Query(params): Query<HashMap<String, String>>,
    Extension(config): Extension<Config>,
) -> impl IntoResponse {
    let query = params
        .get("q")
        .cloned()
        .unwrap_or_default()
        .replace('\n', " ");

    let res = match engines::autocomplete(&config, &query).await {
        Ok(res) => res,
        Err(err) => {
            error!("Autocomplete error for {query}: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json((query, vec![])));
        }
    };

    (StatusCode::OK, Json((query, res)))
}
