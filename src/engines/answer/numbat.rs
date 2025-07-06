use std::{collections::HashSet, sync::LazyLock};

use fend_core::SpanKind;
use maud::{html, PreEscaped};
use numbat::{
    markup::{FormatType, FormattedString, Markup},
    pretty_print::PrettyPrint,
    resolver::CodeSource,
    InterpreterResult, InterpreterSettings, Statement,
};
use tracing::debug;

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

    EngineResponse::answer_html(html! {
        p.answer-query { (query_html) " =" }
        h3 { b { (result_html) } }
    })
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

fn interpret(query: &str) -> Option<(Statement<'_>, Markup)> {
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
        Err(err) => {
            debug!("numbat error: {err}");
            return None;
        }
    };

    let res_markup = match res {
        InterpreterResult::Value(val) => val.pretty_print(),
        InterpreterResult::Continue => return None,
    };
    if res_markup.to_string().trim() == query {
        return None;
    }
    let res_markup = fix_markup(res_markup);

    Some((statements.into_iter().next_back()?, res_markup))
}

fn evaluate_for_autocomplete(query: &str) -> Option<String> {
    let (_statements, res_markup) = interpret(query)?;

    Some(res_markup.to_string().trim().to_string())
}

pub struct NumbatResponse {
    pub query_html: PreEscaped<String>,
    pub result_html: PreEscaped<String>,
}

fn evaluate(query: &str) -> Option<NumbatResponse> {
    let (statement, res_markup) = interpret(query)?;

    let statement_markup = fix_markup(statement.pretty_print());
    let query_html = markup_to_html(statement_markup);
    let result_html = markup_to_html(res_markup);

    Some(NumbatResponse {
        query_html,
        result_html,
    })
}

fn fix_markup(markup: Markup) -> Markup {
    let mut reordered_markup: Vec<FormattedString> = Vec::new();
    const LEFT_SIDE_UNITS: &[&str] = &["$", "€", "£", "¥"];
    for s in markup.0 {
        let FormattedString(_output_type, format_type, content) = s.clone();

        if format_type == FormatType::Unit && LEFT_SIDE_UNITS.contains(&&*content) {
            // remove the last markup if it's whitespace
            if let Some(FormattedString(_, FormatType::Whitespace, _)) = reordered_markup.last() {
                reordered_markup.pop();
            }
            reordered_markup.insert(reordered_markup.len() - 1, s);
        } else {
            reordered_markup.push(s);
        }
    }
    Markup(reordered_markup)
}

fn markup_to_html(markup: Markup) -> PreEscaped<String> {
    let mut html = String::new();
    for FormattedString(_, format_type, content) in markup.0 {
        let class = match format_type {
            FormatType::Value => "answer-calc-constant",
            FormatType::String => "answer-calc-string",
            FormatType::Identifier => "answer-calc-func",
            _ => "",
        };
        if class.is_empty() {
            html.push_str(&html! {(content)}.into_string());
        } else {
            html.push_str(
                &html! {
                    span.(class) { (content) }
                }
                .into_string(),
            );
        }
    }
    PreEscaped(html)
}

pub static NUMBAT_CTX: LazyLock<numbat::Context> = LazyLock::new(|| {
    let mut ctx = numbat::Context::new(numbat::module_importer::BuiltinModuleImporter {});
    let _ = ctx.interpret("use prelude", CodeSource::Internal);
    let _ = ctx.interpret("use units::currencies", CodeSource::Internal);

    ctx.load_currency_module_on_demand(true);

    // a few hardcoded aliases
    // (the lowercase alias code won't work for these because they have prefixes)
    for (alias, canonical) in &[
        ("kb", "kB"),
        ("kib", "KiB"),
        ("mb", "MB"),
        ("mib", "MiB"),
        ("gb", "GB"),
        ("gib", "GiB"),
        ("tb", "TB"),
        ("tib", "TiB"),
        ("pb", "PB"),
        ("pib", "PiB"),
    ] {
        let _ = ctx.interpret(&format!("let {alias} = {canonical}"), CodeSource::Internal);
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
            let _ = ctx.interpret(&format!("let {name_lower} = {name}"), CodeSource::Internal);
        }
    }

    ctx
});
