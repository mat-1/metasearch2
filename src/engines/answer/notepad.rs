use maud::html;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("^(note|text|code) ?(pad|book|edit(or|er)?)$").is_match(&query.query.to_lowercase())
    {
        return EngineResponse::new();
    }

    // This allows pasting styles which is undesired behavior, and the
    // `contenteditable="plaintext-only"` attribute currently only works on Chrome.
    // This should be updated when the attribute becomes available in more browsers
    EngineResponse::answer_html(html! {
        div."answer-notepad" contenteditable {}
    })
}
