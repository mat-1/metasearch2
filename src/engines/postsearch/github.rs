use maud::{html, PreEscaped};
use scraper::{Html, Selector};
use url::Url;

use crate::engines::{answer::regex, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if regex!(r"^https:\/\/github\.com\/[\w-]+\/[\w.-]+$").is_match(&search_result.result.url) {
            return Some(CLIENT.get(search_result.result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(body: &str) -> Option<PreEscaped<String>> {
    let dom = Html::parse_document(body);

    let url_relative = dom
        .select(
            &Selector::parse("main #repository-container-header strong[itemprop='name'] > a")
                .unwrap(),
        )
        .next()?
        .value()
        .attr("href")?;
    let url = format!("https://github.com{url_relative}");

    let embedded_data_script = dom
        .select(&Selector::parse("script[data-target='react-partial.embeddedData']").unwrap())
        .next_back()?
        .inner_html();
    let embedded_data = serde_json::from_str::<serde_json::Value>(&embedded_data_script).ok()?;
    let readme_html = embedded_data
        .get("props")?
        .get("initialPayload")?
        .get("overview")?
        .get("overviewFiles")?
        .as_array()?
        .first()?
        .get("richText")?
        .as_str()?;

    let mut readme_html = ammonia::Builder::default()
        .link_rel(None)
        .add_allowed_classes("div", &["markdown-alert"])
        .add_allowed_classes("p", &["markdown-alert-title"])
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            Url::parse("https://github.com").unwrap(),
        ))
        .clean(readme_html)
        .to_string();

    let readme_dom = Html::parse_fragment(&readme_html);
    let mut readme_element = readme_dom.root_element();

    let mut is_readme_element_pre = false;

    while readme_element.children().count() == 1 {
        // if the readme is wrapped in <article>, remove that
        if let Some(article) = readme_element
            .select(&Selector::parse("article").unwrap())
            .next()
        {
            readme_element = article;
        }
        // useless div
        else if let Some(div) = readme_element
            .select(&Selector::parse("div").unwrap())
            .next()
        {
            readme_element = div;
            // useless pre
        } else if let Some(pre) = readme_element
            .select(&Selector::parse("pre").unwrap())
            .next()
        {
            readme_element = pre;
            is_readme_element_pre = true;
        } else {
            break;
        }
    }

    readme_html = readme_element.inner_html().to_string();

    let title = if let Some(title_el) = readme_dom
        // github wraps their h1s in a <div class="">
        .select(&Selector::parse("div:has(h1)").unwrap())
        .next()
    {
        // if the readme starts with an h1, remove it
        let title_html = title_el.html().trim().to_string();

        if readme_html.starts_with(&title_html) {
            readme_html = readme_html[title_html.len()..].to_string();
        }
        title_el.text().collect::<String>()
    } else {
        dom.select(
            &Selector::parse("main #repository-container-header strong[itemprop='name'] > a")
                .unwrap(),
        )
        .next()?
        .text()
        .collect::<String>()
    };

    Some(html! {
        a href=(url) {
            h1 { (title) }
        }
        @if is_readme_element_pre {
            pre.infobox-github-readme {
                (PreEscaped(readme_html))
            }
        } @else {
            div.infobox-github-readme {
                (PreEscaped(readme_html))
            }
        }
    })
}
