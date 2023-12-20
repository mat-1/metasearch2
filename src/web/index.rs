use axum::{http::header, response::IntoResponse};

pub async fn route() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("index.html"),
    )
}
