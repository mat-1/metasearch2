use std::collections::HashSet;

use fend_core::SpanKind;
use numbat::{
    markup::{FormatType, FormattedString, Markup},
    pretty_print::PrettyPrint,
    resolver::CodeSource,
    InterpreterResult, InterpreterSettings, Statement,
};
use once_cell::sync::Lazy;

use crate::engines::EngineResponse;

pub fn request(query: &str) -> EngineResponse {
    let query = clean_query(query);

    let Some(NumbatResponse {
        query_html,
        result_html,
    }) = evaluate(&query)
    else {
        return EngineResponse::new();
    };

    EngineResponse::answer_html(format!(
        r#"<p class="answer-query">{query_html} =</p>
<h3><b>{result_html}</b></h3>"#
    ))
}

pub fn request_autocomplete(query: &str) -> Vec<String> {
    let mut results = Vec::new();

    let query = clean_query(query);

    if let Some(result) = evaluate_for_autocomplete(&query) {
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

fn is_potential_request(query: &str) -> bool {
    // allow these short constants, they're fine
    if matches!(query.to_lowercase().as_str(), "pi" | "e" | "c") {
        return true;
    }

    // at least 3 characters
    if query.len() < 3 {
        return false;
    }

    // must have numbers
    if !query.chars().any(|c| c.is_numeric()) {
        return false;
    }

    // if it starts and ends with quotes then the person was just searching in
    // quotes and didn't mean to evaluate a string
    if query.starts_with('"')
        && query.ends_with('"')
        && query.chars().filter(|c| *c == '"').count() == 2
    {
        return false;
    }

    true
}

fn interpret(query: &str) -> Option<(Statement, InterpreterResult)> {
    if !is_potential_request(query) {
        return None;
    }

    let mut ctx = NUMBAT_CTX.clone();

    let (statements, res) = match ctx.interpret_with_settings(
        &mut InterpreterSettings {
            print_fn: Box::new(move |_: &Markup| {}),
        },
        query,
        CodeSource::Text,
    ) {
        Ok(r) => r,
        Err(_) => {
            return None;
        }
    };

    Some((statements.into_iter().last()?, res))
}

fn evaluate_for_autocomplete(query: &str) -> Option<String> {
    let (_statements, res) = interpret(query)?;

    let res_markup = match res {
        InterpreterResult::Value(val) => val.pretty_print(),
        InterpreterResult::Continue => return None,
    };

    Some(res_markup.to_string())
}

pub struct NumbatResponse {
    pub query_html: String,
    pub result_html: String,
}

fn evaluate(query: &str) -> Option<NumbatResponse> {
    let (statement, res) = interpret(query)?;

    let res_markup = match res {
        InterpreterResult::Value(val) => val.pretty_print(),
        InterpreterResult::Continue => return None,
    };

    let statement_markup = statement.pretty_print();
    let query_html = markup_to_html(statement_markup);
    let result_html = markup_to_html(res_markup);

    Some(NumbatResponse {
        query_html,
        result_html,
    })
}

fn markup_to_html(markup: Markup) -> String {
    let mut html = String::new();
    for FormattedString(_output_type, format_type, content) in markup.0 {
        let class = match format_type {
            FormatType::Value => "answer-calc-constant",
            FormatType::String => "answer-calc-string",
            FormatType::Identifier => "answer-calc-func",
            _ => "",
        };
        if class.is_empty() {
            html.push_str(&html_escape::encode_safe(&content));
        } else {
            html.push_str(&format!(
                r#"<span class="{class}">{content}</span>"#,
                content = html_escape::encode_safe(&content)
            ));
        }
    }
    html
}

pub static NUMBAT_CTX: Lazy<numbat::Context> = Lazy::new(|| {
    let mut ctx = numbat::Context::new(numbat::module_importer::BuiltinModuleImporter {});
    let _ = ctx.interpret("use prelude", CodeSource::Internal);
    let _ = ctx.interpret("use units::currencies", CodeSource::Internal);

    ctx.load_currency_module_on_demand(true);

    // a few hardcoded aliases
    // (the lowercase alias code won't work for these because they have prefixes)
    for (alias, canonical) in &[
        ("kb", "kB"),
        ("mb", "MB"),
        ("gb", "GB"),
        ("tb", "TB"),
        ("pb", "PB"),
    ] {
        let _ = ctx.interpret(&format!("unit {alias} = {canonical}"), CodeSource::Internal);
    }

    // lowercase aliases (so for example usd and USD are the same unit)

    let mut unit_names = HashSet::new();
    for names in ctx.unit_names() {
        unit_names.extend(names.iter().map(|name| name.to_owned()));
    }

    for name in &unit_names {
        // taken_unit_names.insert(alias_name);
        let name_lower = name.to_lowercase();
        // add every lowercase aliases for every unit as long as that alias isn't
        // already taken
        if !unit_names.contains(&name_lower) {
            let _ = ctx.interpret(&format!("unit {name_lower} = {name}"), CodeSource::Internal);
        }
    }

    ctx
});
