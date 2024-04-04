use eyre::eyre;
use scraper::{Html, Selector};
use serde::Deserialize;
use url::Url;

use crate::engines::{EngineResponse, RequestResponse, CLIENT};

use super::regex;

pub fn request(query: &str) -> RequestResponse {
    let re = regex!(r"^synonyms for\s+(\w+)$");
    let query = match re.captures(query) {
        Some(caps) => caps.get(1).unwrap().as_str(),
        None => return RequestResponse::None,
    }
    .to_lowercase();

    CLIENT
        .get(
            Url::parse(
                format!(
                    "https://www.thesaurus.com/browse/{}",
                    urlencoding::encode(&query.to_lowercase())
                )
                .as_str(),
            )
            .unwrap(),
        )
        .into()
}

#[derive(Debug, Deserialize)]
pub struct ThesaurusResponse {
    /// Example: `silly`
    pub word: String,
    pub items: Vec<ThesaurusItem>,
}

#[derive(Debug, Deserialize)]
pub struct ThesaurusItem {
    /// Example `adjective`
    pub part_of_speech: String,
    /// Example: `absurd, giddy, foolish`
    pub as_in: String,

    pub strongest_matches: Vec<String>,
    pub strong_matches: Vec<String>,
    pub weak_matches: Vec<String>,
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let response = parse_thesaurus_com_response(body)?;

    if response.items.is_empty() {
        return Ok(EngineResponse::new());
    }

    let rendered_html = render_thesaurus_html(response);

    Ok(EngineResponse::answer_html(rendered_html))
}

fn parse_thesaurus_com_response(body: &str) -> eyre::Result<ThesaurusResponse> {
    let dom = Html::parse_document(body);

    let word = dom
        .select(&Selector::parse("h1").unwrap())
        .next()
        .ok_or_else(|| eyre!("No title found"))?
        .text()
        .collect::<String>();

    let card_sel = Selector::parse("[data-type='synonym-and-antonym-card']").unwrap();
    let card_els = dom.select(&card_sel);

    let mut items = Vec::<ThesaurusItem>::new();

    for synonym_and_antonym_card_el in card_els {
        items.push(parse_thesaurus_com_item(synonym_and_antonym_card_el)?);
    }

    Ok(ThesaurusResponse { word, items })
}

fn parse_thesaurus_com_item(
    synonym_and_antonym_card_el: scraper::ElementRef,
) -> eyre::Result<ThesaurusItem> {
    let adjective_as_in_words = synonym_and_antonym_card_el
        .select(&Selector::parse("div:first-child > p").unwrap())
        .next()
        .ok_or_else(|| eyre!("No adjective as in words found"))?
        .text()
        .collect::<String>();
    let (part_of_speech, as_in) = adjective_as_in_words
        .split_once(" as in ")
        .ok_or_else(|| eyre!("No 'as in' found"))?;
    let part_of_speech = part_of_speech.trim().to_owned();
    let as_in = as_in.trim().to_owned();

    let matches_container_el = synonym_and_antonym_card_el
        .select(&Selector::parse("div:nth-child(2) > div:nth-child(2)").unwrap())
        .next()
        .ok_or_else(|| eyre!("No matches container found"))?;

    let mut strongest_matches = Vec::<String>::new();
    let mut strong_matches = Vec::<String>::new();
    let mut weak_matches = Vec::<String>::new();

    for match_el in matches_container_el.select(&Selector::parse("div").unwrap()) {
        let match_type = match_el
            .select(&Selector::parse("p").unwrap())
            .next()
            .ok_or_else(|| eyre!("No match type found"))?
            .text()
            .collect::<String>();
        let match_type = match_type
            .split(' ')
            .next()
            .ok_or_else(|| eyre!("No match type found"))?;

        let matches = match_el
            .select(&Selector::parse("a").unwrap())
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<String>>();

        match match_type {
            "Strongest" => {
                strongest_matches = matches;
            }
            "Strong" => {
                strong_matches = matches;
            }
            "Weak" => {
                weak_matches = matches;
            }
            _ => {
                eprintln!("Unknown thesaurus match type: {match_type}");
            }
        }
    }

    Ok(ThesaurusItem {
        part_of_speech,
        as_in,
        strongest_matches,
        strong_matches,
        weak_matches,
    })
}

fn render_thesaurus_html(ThesaurusResponse { word, items }: ThesaurusResponse) -> String {
    let mut html = String::new();

    html.push_str(&format!(
        "<h2 class=\"answer-thesaurus-word\"><a href=\"https://www.thesaurus.com/browse/{word}\">{word}</a></h2>",
        word = html_escape::encode_safe(&word)
    ));

    html.push_str("<div class=\"answer-thesaurus-items\">");
    for item in items {
        html.push_str("<div class=\"answer-thesaurus-item\">");
        html.push_str(&render_thesaurus_item_html(item));
        html.push_str("</div>");
    }
    html.push_str("</div>");

    html
}

fn render_thesaurus_item_html(
    ThesaurusItem {
        part_of_speech,
        as_in,
        strongest_matches,
        strong_matches,
        weak_matches,
    }: ThesaurusItem,
) -> String {
    let mut html = String::new();

    html.push_str(&format!(
        "<span class=\"answer-thesaurus-word-description\"><span class=\"answer-thesaurus-part-of-speech\">{part_of_speech}</span>, as in <span class=\"answer-thesaurus-as-in\">{as_in}</span></span>",
        part_of_speech = html_escape::encode_safe(&part_of_speech.to_lowercase()),
        as_in = html_escape::encode_safe(&as_in)
    ));

    let render_matches = |matches: Vec<String>, strength: &str| {
        if matches.is_empty() {
            return String::new();
        }

        let mut html = String::new();

        html.push_str(&format!(
            "<div class=\"answer-thesaurus-{strength_id}\">",
            strength_id = html_escape::encode_safe(&strength.to_lowercase().replace(' ', "-"))
        ));

        html.push_str(&format!(
            "<h3 class=\"answer-thesaurus-category-title\">{strength} {match_or_matches}</h3>",
            strength = html_escape::encode_safe(&strength),
            match_or_matches = if matches.len() == 1 {
                "match"
            } else {
                "matches"
            }
        ));
        html.push_str("<ul class=\"answer-thesaurus-list\">");
        for synonym in matches {
            html.push_str(&format!(
                "<li><a href=\"https://www.thesaurus.com/browse/{synonym}\">{synonym}</a></li>",
                synonym = html_escape::encode_safe(&synonym)
            ));
        }
        html.push_str("</ul>");

        html.push_str("</div>");

        html
    };

    html.push_str(&render_matches(strongest_matches, "Strongest"));
    html.push_str(&render_matches(strong_matches, "Strong"));
    html.push_str(&render_matches(weak_matches, "Weak"));

    html
}
