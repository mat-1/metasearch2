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

    let all_headers_html = html! {
        br;
        details {
            summary { "All headers" }
            @for (header, value) in query.request_headers.iter() {
                div {
                    b { (header) } ": " (value)
                }
            }
        }
    };

    EngineResponse::answer_html(if let Some(user_agent) = user_agent {
        html! {
            h3 { b { (user_agent) } }
            (all_headers_html)
        }
    } else {
        html! {
            "You don't have a user agent"
            (all_headers_html)
        }
    })
}
