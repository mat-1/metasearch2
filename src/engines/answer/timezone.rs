use std::{cell::Cell, sync::LazyLock};

use fend_core::SpanKind;

use crate::engines::EngineResponse;

use super::regex;

pub fn request(query: &str) -> EngineResponse {
    let query = clean_query(query.to_string());

    let Some(result_html) = evaluate(&query, true) else {
        return EngineResponse::new();
    };

    EngineResponse::answer_html(format!(
        r#"<p class="answer-calc-query">{query} =</p>
<h3><b>{result_html}</b></h3>"#,
        query = html_escape::encode_text(&query),
    ))
}
