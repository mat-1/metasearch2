use fend_core::SpanKind;
use maud::{html, PreEscaped};
use std::sync::{atomic::AtomicU32, atomic::Ordering, LazyLock};

use crate::engines::EngineResponse;

use super::regex;

pub fn request(query: &str) -> EngineResponse {
    let query = clean_query(query);

    let Some(result_html) = evaluate_to_html(&query, true) else {
        return EngineResponse::new();
    };

    EngineResponse::answer_html(html! {
        p.answer-query { (query) " =" }
        h3 { b { (result_html) } }
    })
}

pub fn request_autocomplete(query: &str) -> Vec<String> {
    let mut results = Vec::new();

    let query = clean_query(query);

    if let Some(result) = evaluate_to_plaintext(&query, false) {
        results.push(format!("= {result}"));
    }

    results
}

fn clean_query(query: &str) -> String {
    query.strip_suffix('=').unwrap_or(query).trim().to_string()
}

#[derive(Debug)]
pub struct Span {
    pub text: String,
    pub kind: SpanKind,
}

fn evaluate_to_plaintext(query: &str, html: bool) -> Option<String> {
    let spans = evaluate_into_spans(query, html);
    if spans.is_empty() {
        return None;
    }

    Some(
        spans
            .iter()
            .map(|span| span.text.clone())
            .collect::<String>(),
    )
}

fn evaluate_to_html(query: &str, html: bool) -> Option<PreEscaped<String>> {
    let spans = evaluate_into_spans(query, html);
    if spans.is_empty() {
        return None;
    }

    let mut result_html = String::new();
    for span in &spans {
        let class = match span.kind {
            fend_core::SpanKind::Number
            | fend_core::SpanKind::Boolean
            | fend_core::SpanKind::Date => "answer-calc-constant",
            fend_core::SpanKind::String => "answer-calc-string",
            _ => "",
        };
        if class.is_empty() {
            result_html.push_str(&html! { (span.text) }.into_string());
        } else {
            result_html.push_str(
                &html! {
                    span.(class) {
                        (span.text)
                    }
                }
                .into_string(),
            );
        }
    }

    // if the result was a single hex number then we add the decimal equivalent
    // below
    if spans.len() == 1
        && spans[0].kind == fend_core::SpanKind::Number
        && spans[0].text.starts_with("0x")
    {
        let hex = spans[0].text.trim_start_matches("0x");
        if let Ok(num) = u64::from_str_radix(hex, 16) {
            result_html.push_str(
                &html! {
                    span.answer-comment { " = " (num) }
                }
                .into_string(),
            );
        }
    }

    Some(PreEscaped(result_html))
}

pub static FEND_CTX: LazyLock<fend_core::Context> = LazyLock::new(|| {
    let mut context = fend_core::Context::new();

    // make lowercase f and c work
    context.define_custom_unit_v1("f", "f", "°F", &fend_core::CustomUnitAttribute::Alias);
    context.define_custom_unit_v1("c", "c", "°C", &fend_core::CustomUnitAttribute::Alias);

    context.define_custom_unit_v1(
        "mb",
        "mbs",
        "megabyte",
        &fend_core::CustomUnitAttribute::Alias,
    );
    context.define_custom_unit_v1(
        "gb",
        "gbs",
        "gigabyte",
        &fend_core::CustomUnitAttribute::Alias,
    );
    context.define_custom_unit_v1(
        "tb",
        "tbs",
        "terabyte",
        &fend_core::CustomUnitAttribute::Alias,
    );
    context.define_custom_unit_v1(
        "pb",
        "pbs",
        "petabyte",
        &fend_core::CustomUnitAttribute::Alias,
    );

    // make random work
    context.set_random_u32_fn(rand::random::<u32>);

    fend_core::evaluate("ord=(x: x to codepoint)", &mut context).unwrap();
    fend_core::evaluate("chr=(x: x to character)", &mut context).unwrap();

    context
});

struct Interrupter {
    invocations_left: AtomicU32,
}

impl fend_core::Interrupt for Interrupter {
    fn should_interrupt(&self) -> bool {
        let v = self.invocations_left.load(Ordering::Relaxed);

        if v == 0 {
            return true;
        }

        self.invocations_left.store(v - 1, Ordering::Relaxed);
        false
    }
}

fn evaluate_into_spans(query: &str, multiline: bool) -> Vec<Span> {
    // fend incorrectly triggers on these often
    {
        // at least 3 characters and not one of the short constants
        if query.len() < 3 && !matches!(query.to_lowercase().as_str(), "pi" | "e" | "c") {
            return vec![];
        }

        // probably a query operator thing or a url, fend evaluates these but it
        // shouldn't
        if regex!("^[a-z]{2,}:").is_match(query) {
            return vec![];
        }

        // if it starts and ends with quotes then the person was just searching in
        // quotes and didn't mean to evaluate a string
        if query.starts_with('"')
            && query.ends_with('"')
            && query.chars().filter(|c| *c == '"').count() == 2
        {
            return vec![];
        }
    }

    let mut context = FEND_CTX.clone();
    if multiline {
        // this makes it generate slightly nicer outputs for some queries like 2d6
        context.set_output_mode_terminal();
    }

    // avoids stackoverflows and queries that take too long
    // examples:
    // - Y = (\f. (\x. f x x)) (\x. f x x); Y(Y)
    // - 10**100000000
    let interrupt = Interrupter {
        invocations_left: AtomicU32::new(1000),
    };
    let Ok(result) = fend_core::evaluate_with_interrupt(query, &mut context, &interrupt) else {
        return vec![];
    };
    let main_result = result.get_main_result();
    if main_result == query {
        return vec![];
    }

    let res = result
        .get_main_result_spans()
        .filter(|span| !span.string().is_empty())
        .map(|span| Span {
            text: span.string().to_string(),
            kind: span.kind(),
        })
        .collect::<Vec<_>>();

    if let Some(first) = res.first() {
        if first.kind == SpanKind::Other && first.text.starts_with("\\") {
            // false positive, can happen if you search like "a: b"
            return vec![];
        }
    }

    res
}
