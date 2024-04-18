use maud::html;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("^what('s|s| is) my ip").is_match(&query.query.to_lowercase()) {
        return EngineResponse::new();
    }

    let ip = &query.ip;

    EngineResponse::answer_html(html! {
        h3 { b { (ip) } }
    })
}
