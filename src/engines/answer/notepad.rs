use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("(note|text|code) ?(pad|book|edit(or|er)?)").is_match(&query.query.to_lowercase()) {
        return EngineResponse::new();
    }

    EngineResponse::answer_html(
        r#"<div contenteditable id='notepad' placeholder='Notes' style='width:100%;color:white;outline:none;min-height:4em;font-size:12px;'></div>"#.to_string()
    )
}
