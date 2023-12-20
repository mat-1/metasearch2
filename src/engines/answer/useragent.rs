use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(_client: &reqwest::Client, query: &SearchQuery) -> EngineResponse {
    if !regex!("^what('s|s| is) my (user ?agent|ua)|ua|user ?agent$")
        .is_match(&query.query.to_lowercase())
    {
        return EngineResponse::new();
    }

    let user_agent = query.request_headers.get("user-agent");

    EngineResponse::answer_html(if let Some(user_agent) = user_agent {
        format!("Your user agent is <b>{user_agent}</b>")
    } else {
        format!("You don't have a user agent")
    })
}
