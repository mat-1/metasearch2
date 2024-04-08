use crate::engines::answer::regex;
use crate::engines::{EngineResponse, RequestResponse, CLIENT};
use html_escape::encode_safe;
use serde::Deserialize;
use serde_json::{json, Value};

// this may look weird, but trust the process
const QUERY_URL: &str = "https://79frdp12pn-dsn.algolia.net/1/indexes/*/queries?x-algolia-api-key=175588f6e5f8319b27702e4cc4013561&x-algolia-application-id=79FRDP12PN";

pub fn request(query: &str) -> RequestResponse {
    let re = regex!(r"^(rt|rottentomatoes|rotten tomatoes)\s+(.+)$");
    let query = match re.captures(query) {
        Some(caps) => caps.get(2).unwrap().as_str(),
        None => return RequestResponse::None,
    }
    .to_lowercase();

    CLIENT
        .post(QUERY_URL)
        .body(json!({"requests":[{"indexName":"content_rt","query": query}]}).to_string())
        .into()
}

pub struct RottenTomatoesResponse {
    // 0-100%
    pub tomatometer: u8,
    // 0-100%
    pub audience_score: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Hit {
    title: String,
    vanity: String,
    description: String,
    release_year: Option<u16>,
    run_time: Option<u16>,
    genres: Vec<String>,
    rotten_tomatoes: HitRottenTomatoes,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HitRottenTomatoes {
    audience_score: u8,
    // aka tomatoscore
    critics_score: u8,
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let res: Value = serde_json::from_str(body)?;
    let mut hit = None;
    for i in 0..res["results"][0]["hits"].as_array().unwrap().len() {
        if let Ok(h) = serde_json::from_value::<Hit>(res["results"][0]["hits"][i].clone()) {
            hit = Some(h);
            break;
        };
    }
    let Some(hit) = hit else {
        return Ok(EngineResponse::answer_html(
            r#"<span style="color: red">Error: Show not found</span>"#.to_string(),
        ));
    };

    let rendered_html = render_rottentomatoes_html(hit);

    Ok(EngineResponse::answer_html(rendered_html))
}

fn render_rottentomatoes_html(hit: Hit) -> String {
    let mut html = String::new();
    html.push_str(&format!("<h2>{title}</h2>", title = hit.title));
    html.push_str(&format!(
        "<span>Release year: {release_year}</span><br>",
        release_year = hit.release_year.unwrap_or_default()
    ));
    html.push_str(&format!(
        "<span>Genre(s): {genres}</span><br>",
        genres = hit.genres.join(", ")
    ));
    html.push_str(&format!(
        "<span>Length: {length_mins}mins</span><br><br>",
        length_mins = hit.run_time.unwrap_or_default()
    ));
    html.push_str(&format!(
        "<span>Tomatometer: </span>{tomatometer}<br>",
        tomatometer = colorize_percentage(hit.rotten_tomatoes.critics_score)
    ));
    html.push_str(&format!(
        "<span>Audience score: </span>{audience_score}<br><br>",
        audience_score = colorize_percentage(hit.rotten_tomatoes.audience_score)
    ));
    html.push_str(&format!(
        "Description:<br>{description}<br><br>",
        description = encode_safe(&hit.description)
    ));
    html.push_str(&format!(
        r#"<a href="https://rottentomatoes.com/m/{vanity}">View on Rotten Tomatoes...</a>"#,
        vanity = hit.vanity
    ));

    html
}

fn colorize_percentage(perc: u8) -> String {
    let color_hex = match perc {
        0..=40 => "8a0000",
        41..=60 => "cf5813",
        61..=70 => "cf9013",
        71..=80 => "cfbf13",
        81..=90 => "89b814",
        91..=100 => "5eff00",

        101..=u8::MAX => unreachable!(),
    }
    .to_string();

    format!(r#"<span style="color: #{color_hex}">{perc}%</span>"#)
}
