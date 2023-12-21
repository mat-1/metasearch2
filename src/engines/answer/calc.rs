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

pub fn request_autocomplete(query: &str) -> Vec<String> {
    let mut results = Vec::new();

    let query = clean_query(query.to_string());

    if let Some(result) = evaluate(&query, false) {
        results.push(format!("{query}={result}"));
    }

    results
}

fn clean_query(query: String) -> String {
    query.strip_suffix('=').unwrap_or(&query).trim().to_string()
}

fn evaluate(query: &str, html: bool) -> Option<String> {
    // at least 3 characters and not one of the short constants
    if query.len() < 3 && !matches!(query.to_lowercase().as_str(), "pi" | "e" | "c") {
        return None;
    }

    // probably a query operator thing or a url, fend evaluates these but it shouldn't
    if regex!("^[a-z]{2,}:").is_match(query) {
        return None;
    }

    let mut context = fend_core::Context::new();

    // make lowercase f and c work
    context.define_custom_unit_v1("f", "f", "°F", &fend_core::CustomUnitAttribute::Alias);
    context.define_custom_unit_v1("c", "c", "°C", &fend_core::CustomUnitAttribute::Alias);
    // make random work
    context.set_random_u32_fn(rand::random::<u32>);
    if html {
        // this makes it generate slightly nicer outputs for some queries like 2d6
        context.set_output_mode_terminal();
    }

    let Ok(result) = fend_core::evaluate(query, &mut context) else {
        return None;
    };
    let main_result = result.get_main_result();
    if main_result == query {
        return None;
    }

    if !html {
        return Some(main_result.to_string());
    }

    let mut result_html = String::new();
    for span in result.get_main_result_spans() {
        let class = match span.kind() {
            fend_core::SpanKind::Number
            | fend_core::SpanKind::Boolean
            | fend_core::SpanKind::Date => "answer-calc-constant",
            fend_core::SpanKind::String => "answer-calc-string",
            _ => "",
        };
        if !class.is_empty() {
            result_html.push_str(&format!(
                r#"<span class="{class}">{text}</span>"#,
                text = html_escape::encode_text(span.string())
            ));
        } else {
            result_html.push_str(&html_escape::encode_text(span.string()));
        }
    }

    Some(result_html)
}
