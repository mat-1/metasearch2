use crate::engines::answer::regex;
use crate::engines::{EngineResponse, RequestResponse, CLIENT};
use html_escape::encode_safe;
use levenshtein::levenshtein;
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

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Hit {
    title: String,
    vanity: String,
    description: Option<String>,
    release_year: Option<u16>,
    run_time: Option<u16>,
    genres: Option<Vec<String>>,
    rotten_tomatoes: Option<HitRottenTomatoes>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct HitRottenTomatoes {
    audience_score: Option<u8>,
    // aka tomatoscore
    critics_score: Option<u8>,
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let res: Value = serde_json::from_str(body)?;
    let hits: Vec<Hit> = serde_json::from_value(res["results"][0]["hits"].clone()).unwrap();
    
    if hits.is_empty() {
        return Ok(EngineResponse::answer_html(
            r#"<span style="color: red">Error: Show not found</span>"#.to_string(),
        ));
    };

    let query = res["results"][0]["query"].as_str().unwrap();
    let (_, _, hit) = hits
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, h)| (i, levenshtein(&h.title, query), h))
        .max_by_key(|(_, dist, hit)| {
            usize::MAX / 2
                - dist 
                // prefer shows with ratings
                + hit.rotten_tomatoes.is_some() as usize * 2
                // prefer shows with description
                + hit.description.is_some() as usize
        })
        .unwrap();
    let rendered_html = render_rottentomatoes_html(&hit);

    Ok(EngineResponse::answer_html(rendered_html))
}

fn render_rottentomatoes_html(hit: &Hit) -> String {
    let mut html = String::new();
    
    html.push_str(&format!("<h2>{title}</h2>", title = hit.title));
    
    if let Some(release_year) = hit.release_year {
        html.push_str(&format!("<span>Release year: {release_year}</span><br>"));
    } else {
        html.push_str(r#"<span style="color: gray">Release year: Unknown</span><br><br>"#);
    }
    
    if let Some(genres) = &hit.genres {
        html.push_str(&format!(
            "<span>Genre(s): {genres}</span><br>",
            genres = genres.join(", ")
        ));
    } else {
        html.push_str(r#"<span style="color: gray">Genre(s): Unknown</span><br><br>"#);
    }
    
    if let Some(runtime) = hit.run_time {
        html.push_str(&format!("<span>Length: {runtime}mins</span><br><br>"));
    } else {
        html.push_str(r#"<span style="color: gray">Length: Unknown</span><br><br>"#);
    }

    if let Some(rt) = &hit.rotten_tomatoes {
        html.push_str(&format!(
            "<span>Tomatometer: </span>{tomatometer}<br>",
            tomatometer = rt
                .critics_score
                .map_or_else(|| "-".to_string(), |score| colorize_percentage(score))
        ));
        html.push_str(&format!(
            "<span>Audience score: </span>{audience_score}<br><br>",
            audience_score = rt
                .audience_score
                .map_or_else(|| "-".to_string(), |score| colorize_percentage(score))
        ));
    } else {
        html.push_str(r#"<span style="color: gray">(No reviews yet)</span><br><br>"#);
    }
    
    if let Some(description) = &hit.description {
        html.push_str(&format!(
            "Description:<br>{description}<br><br>",
            description = encode_safe(description)
        ));
    } else {
        html.push_str(r#"Description:<br><span color="color: gray">No description</span><br><br>"#);
    }
    
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
