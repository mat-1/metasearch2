use scraper::{Html, Selector};
use url::Url;

use crate::engines::{answer::regex, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if regex!(r"^https:\/\/(stackoverflow\.com|serverfault\.com|superuser\.com|\w{1,}\.stackexchange\.com)\/questions\/\d+")
            .is_match(&search_result.url)
        {
            return Some(CLIENT.get(search_result.url.as_str()));
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

    let base_url = dom
        .select(&Selector::parse("link[rel=canonical]").unwrap())
        .next()?
        .value()
        .attr("href")?;
    let url = Url::join(
        &Url::parse(base_url).unwrap(),
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

    let answer_html = ammonia::Builder::default()
        .url_relative(ammonia::UrlRelative::RewriteWithBase(url.clone()))
        .clean(&answer_html)
        .to_string();

    let url = format!("{url}#{answer_id}");

    Some(format!(
        r#"<a href="{url}"><h2>{title}</h2></a>
<div class="infobox-stackexchange-answer">{answer_html}</div>"#,
        url = html_escape::encode_quoted_attribute(&url.to_string()),
        title = html_escape::encode_safe(&title),
    ))
}
