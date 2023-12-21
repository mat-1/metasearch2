use reqwest::Url;
use scraper::{Html, Selector};

use crate::engines::{Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if search_result
            .url
            .starts_with("https://stackoverflow.com/questions/")
        {
            return Some(CLIENT.get(search_result.url.as_str()).header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
            ));
        }
    }

    None
}

pub fn parse_response(body: &str) -> Option<String> {
    let dom = Html::parse_document(body);

    let title = dom
        .select(&Selector::parse("h1").unwrap())
        .next()?
        .text()
        .collect::<String>();
    let url = Url::join(
        &Url::parse("https://stackoverflow.com").unwrap(),
        dom.select(&Selector::parse(".question-hyperlink").unwrap())
            .next()?
            .value()
            .attr("href")?,
    )
    .ok()?;

    let answer_query = Selector::parse("div.answer.accepted-answer").unwrap();

    let answer = dom.select(&answer_query).next()?;
    let answer_id = answer.value().attr("data-answerid")?;
    let answer_html = answer
        .select(&Selector::parse("div.answercell > div.js-post-body").unwrap())
        .next()?
        .html()
        .to_string();

    let url = format!("{url}#{answer_id}");

    Some(format!(
        r#"<a href="{url}"><h2>{title}</h2></a>
<div class="infobox-stackoverflow-answer">{answer_html}</div>"#,
        url = html_escape::encode_quoted_attribute(&url.to_string()),
        title = html_escape::encode_text(&title),
    ))
}
