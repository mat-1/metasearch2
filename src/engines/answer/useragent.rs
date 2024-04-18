use maud::html;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("^(what('s|s| is) my (user ?agent|ua)|ua|user ?agent)$")
        .is_match(&query.query.to_lowercase())
    {
        return EngineResponse::new();
    }

    let user_agent = query.request_headers.get("user-agent");

    EngineResponse::answer_html(if let Some(user_agent) = user_agent {
        html! {
            h3 { b { (user_agent) } }
        }
    } else {
        html! { "You don't have a user agent" }
    })
}
