use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Display},
    net::IpAddr,
    ops::Deref,
    str::FromStr,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use eyre::bail;
use futures::future::join_all;
use maud::PreEscaped;
use reqwest::{header::HeaderMap, RequestBuilder};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info};

mod macros;
mod ranking;
use crate::{
    config::Config, engine_autocomplete_requests, engine_image_requests,
    engine_postsearch_requests, engine_requests, engines,
};

pub mod answer;
pub mod postsearch;
pub mod search;

engines! {
    // search
    Google = "google",
    GoogleScholar = "google_scholar",
    Bing = "bing",
    Brave = "brave",
    Marginalia = "marginalia",
    RightDao = "rightdao",
    Stract = "stract",
    Yep = "yep",
    // answer
    Dictionary = "dictionary",
    Fend = "fend",
    Ip = "ip",
    Notepad = "notepad",
    ColorPicker = "colorpicker",
    Numbat = "numbat",
    Thesaurus = "thesaurus",
    Timezone = "timezone",
    Useragent = "useragent",
    Wikipedia = "wikipedia",
    // post-search
    DocsRs = "docs_rs",
    GitHub = "github",
    Mdn = "mdn",
    MinecraftWiki = "minecraft_wiki",
    StackExchange = "stackexchange",
}

engine_requests! {
    // search
    Bing => search::bing::request, parse_response,
    Brave => search::brave::request, parse_response,
    GoogleScholar => search::google_scholar::request, parse_response,
    Google => search::google::request, parse_response,
    Marginalia => search::marginalia::request, parse_response,
    RightDao => search::rightdao::request, parse_response,
    Stract => search::stract::request, parse_response,
    Yep => search::yep::request, parse_response,
    // answer
    Dictionary => answer::dictionary::request, parse_response,
    Fend => answer::fend::request, None,
    Ip => answer::ip::request, None,
    Notepad => answer::notepad::request, None,
    ColorPicker => answer::colorpicker::request, None,
    Numbat => answer::numbat::request, None,
    Thesaurus => answer::thesaurus::request, parse_response,
    Timezone => answer::timezone::request, None,
    Useragent => answer::useragent::request, None,
    Wikipedia => answer::wikipedia::request, parse_response,
}

engine_autocomplete_requests! {
    Google => search::google::request_autocomplete, parse_autocomplete_response,
    Fend => answer::fend::request_autocomplete, None,
    Numbat => answer::numbat::request_autocomplete, None,
}

engine_postsearch_requests! {
    DocsRs => postsearch::docs_rs::request, parse_response,
    GitHub => postsearch::github::request, parse_response,
    Mdn => postsearch::mdn::request, parse_response,
    MinecraftWiki => postsearch::minecraft_wiki::request, parse_response,
    StackExchange => postsearch::stackexchange::request, parse_response,
}

engine_image_requests! {
    Google => search::google::request_images, parse_images_response,
    Bing => search::bing::request_images, parse_images_response,
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl<'de> Deserialize<'de> for Engine {
    fn deserialize<D>(deserializer: D) -> Result<Engine, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Engine::from_str(&s).map_err(|_| serde::de::Error::custom(format!("invalid engine '{s}'")))
    }
}

pub struct SearchQuery {
    pub query: String,
    pub tab: SearchTab,
    pub request_headers: HashMap<String, String>,
    pub ip: String,
    /// The config is part of the query so it's possible to make a query with a
    /// custom config.
    pub config: Arc<Config>,
}

impl Deref for SearchQuery {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTab {
    #[default]
    All,
    Images,
}
impl FromStr for SearchTab {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "images" => Ok(Self::Images),
            _ => Err(()),
        }
    }
}
impl Display for SearchTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Images => write!(f, "images"),
        }
    }
}

pub enum RequestResponse {
    None,
    Http(reqwest::RequestBuilder),
    Instant(EngineResponse),
}
impl From<reqwest::RequestBuilder> for RequestResponse {
    fn from(req: reqwest::RequestBuilder) -> Self {
        Self::Http(req)
    }
}
impl From<EngineResponse> for RequestResponse {
    fn from(res: EngineResponse) -> Self {
        Self::Instant(res)
    }
}

pub enum RequestAutocompleteResponse {
    Http(Box<reqwest::RequestBuilder>),
    Instant(Vec<String>),
}
impl From<reqwest::RequestBuilder> for RequestAutocompleteResponse {
    fn from(req: reqwest::RequestBuilder) -> Self {
        Self::Http(Box::new(req))
    }
}
impl From<Vec<String>> for RequestAutocompleteResponse {
    fn from(res: Vec<String>) -> Self {
        Self::Instant(res)
    }
}

pub struct HttpResponse {
    pub res: reqwest::Response,
    pub body: String,
    pub config: Arc<Config>,
}

impl<'a> From<&'a HttpResponse> for &'a str {
    fn from(res: &'a HttpResponse) -> Self {
        &res.body
    }
}

impl From<HttpResponse> for reqwest::Response {
    fn from(res: HttpResponse) -> Self {
        res.res
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineSearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug)]
pub struct EngineFeaturedSnippet {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct EngineResponse {
    pub search_results: Vec<EngineSearchResult>,
    pub featured_snippet: Option<EngineFeaturedSnippet>,
    pub answer_html: Option<PreEscaped<String>>,
    pub infobox_html: Option<PreEscaped<String>>,
}

#[derive(Default)]
pub struct EngineImagesResponse {
    pub image_results: Vec<EngineImageResult>,
}

impl EngineResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn answer_html(html: PreEscaped<String>) -> Self {
        Self {
            answer_html: Some(html),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn infobox_html(html: PreEscaped<String>) -> Self {
        Self {
            infobox_html: Some(html),
            ..Default::default()
        }
    }
}

impl EngineImagesResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineImageResult {
    pub image_url: String,
    pub page_url: String,
    pub title: String,
    pub width: u64,
    pub height: u64,
}

#[derive(Debug)]
pub enum EngineProgressUpdate {
    Requesting,
    Downloading,
    Parsing,
    Done,
    Error(String),
}

#[derive(Debug)]
pub enum ProgressUpdateData {
    Engine {
        engine: Engine,
        update: EngineProgressUpdate,
    },
    Response(ResponseForTab),
    PostSearchInfobox(Infobox),
}

#[derive(Debug)]
pub struct ProgressUpdate {
    pub data: ProgressUpdateData,
    pub time_ms: u64,
}

impl ProgressUpdate {
    #[must_use]
    pub fn new(data: ProgressUpdateData, start_time: Instant) -> Self {
        Self {
            data,
            time_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

async fn make_request(
    request: RequestBuilder,
    engine: Engine,
    query: &SearchQuery,
    send_engine_progress_update: impl Fn(Engine, EngineProgressUpdate),
) -> eyre::Result<HttpResponse> {
    send_engine_progress_update(engine, EngineProgressUpdate::Requesting);

    let mut res = request.send().await?;

    send_engine_progress_update(engine, EngineProgressUpdate::Downloading);

    let mut body_bytes = Vec::new();
    while let Some(chunk) = res.chunk().await? {
        body_bytes.extend_from_slice(&chunk);
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    send_engine_progress_update(engine, EngineProgressUpdate::Parsing);

    let http_response = HttpResponse {
        res,
        body,
        config: query.config.clone(),
    };
    Ok(http_response)
}

async fn make_requests(
    query: &SearchQuery,
    progress_tx: &mpsc::UnboundedSender<ProgressUpdate>,
    start_time: Instant,
    send_engine_progress_update: &impl Fn(Engine, EngineProgressUpdate),
) -> eyre::Result<()> {
    let mut requests = Vec::new();
    for &engine in Engine::all() {
        let engine_config = query.config.engines.get(engine);
        if !engine_config.enabled {
            continue;
        }

        requests.push(async move {
            let request_response = engine.request(query);

            let response = match request_response {
                RequestResponse::Http(request) => {
                    let http_response =
                        match make_request(request, engine, query, send_engine_progress_update)
                            .await
                        {
                            Ok(http_response) => http_response,
                            Err(e) => {
                                send_engine_progress_update(
                                    engine,
                                    EngineProgressUpdate::Error(e.to_string()),
                                );
                                return Err(e);
                            }
                        };

                    let response = match engine.parse_response(&http_response) {
                        Ok(response) => response,
                        Err(e) => {
                            error!("parse error for {engine}: {e}");
                            send_engine_progress_update(
                                engine,
                                EngineProgressUpdate::Error(e.to_string()),
                            );
                            return Err(e);
                        }
                    };

                    send_engine_progress_update(engine, EngineProgressUpdate::Done);

                    response
                }
                RequestResponse::Instant(response) => response,
                RequestResponse::None => EngineResponse::new(),
            };

            Ok((engine, response))
        });
    }

    let mut response_futures = Vec::new();
    for request in requests {
        response_futures.push(request);
    }

    let mut responses = HashMap::new();
    for response_result in join_all(response_futures).await {
        let response_result: eyre::Result<_> = response_result; // this line is necessary to make type inference work
        if let Ok((engine, response)) = response_result {
            responses.insert(engine, response);
        }
    }

    let response = ranking::merge_engine_responses(query.config.clone(), responses);
    let has_infobox = response.infobox.is_some();
    progress_tx.send(ProgressUpdate::new(
        ProgressUpdateData::Response(ResponseForTab::All(response.clone())),
        start_time,
    ))?;

    if !has_infobox {
        // post-search

        let mut postsearch_requests = Vec::new();
        for &engine in Engine::all() {
            let engine_config = query.config.engines.get(engine);
            if !engine_config.enabled {
                continue;
            }

            if let Some(request) = engine.postsearch_request(&response) {
                postsearch_requests.push(async move {
                    let response = match request.send().await {
                        Ok(mut res) => {
                            let mut body_bytes = Vec::new();
                            while let Some(chunk) = res.chunk().await? {
                                body_bytes.extend_from_slice(&chunk);
                            }
                            let body = String::from_utf8_lossy(&body_bytes).to_string();

                            let http_response = HttpResponse {
                                res,
                                body,
                                config: query.config.clone(),
                            };
                            engine.postsearch_parse_response(&http_response)
                        }
                        Err(e) => {
                            error!("postsearch request error: {e}");
                            None
                        }
                    };
                    Ok((engine, response))
                });
            }
        }

        let mut postsearch_response_futures = Vec::new();
        for request in postsearch_requests {
            postsearch_response_futures.push(request);
        }

        let postsearch_responses_result: eyre::Result<HashMap<_, _>> =
            join_all(postsearch_response_futures)
                .await
                .into_iter()
                .collect();
        let postsearch_responses = postsearch_responses_result?;

        for (engine, response) in postsearch_responses {
            if let Some(html) = response {
                progress_tx.send(ProgressUpdate::new(
                    ProgressUpdateData::PostSearchInfobox(Infobox { html, engine }),
                    start_time,
                ))?;
                // break so we don't send multiple infoboxes
                break;
            }
        }
    }

    Ok(())
}

async fn make_image_requests(
    query: &SearchQuery,
    progress_tx: &mpsc::UnboundedSender<ProgressUpdate>,
    start_time: Instant,
    send_engine_progress_update: &impl Fn(Engine, EngineProgressUpdate),
) -> eyre::Result<()> {
    let mut requests = Vec::new();
    for &engine in Engine::all() {
        let engine_config = query.config.engines.get(engine);
        if !engine_config.enabled {
            continue;
        }

        requests.push(async move {
            let request_response = engine.request_images(query);

            let response = match request_response {
                RequestResponse::Http(request) => {
                    let http_response =
                        make_request(request, engine, query, send_engine_progress_update).await?;

                    let response = match engine.parse_images_response(&http_response) {
                        Ok(response) => response,
                        Err(e) => {
                            error!("parse error for {engine} (images): {e}");
                            EngineImagesResponse::new()
                        }
                    };

                    send_engine_progress_update(engine, EngineProgressUpdate::Done);

                    response
                }
                RequestResponse::Instant(_) => {
                    error!("unexpected instant response for image request");
                    EngineImagesResponse::new()
                }
                RequestResponse::None => EngineImagesResponse::new(),
            };

            Ok((engine, response))
        });
    }

    let mut response_futures = Vec::new();
    for request in requests {
        response_futures.push(request);
    }

    let responses_result: eyre::Result<HashMap<_, _>> =
        join_all(response_futures).await.into_iter().collect();
    let responses = responses_result?;

    let response = ranking::merge_images_responses(query.config.clone(), responses);
    progress_tx.send(ProgressUpdate::new(
        ProgressUpdateData::Response(ResponseForTab::Images(response.clone())),
        start_time,
    ))?;

    Ok(())
}

#[tracing::instrument(fields(query = %query.query), skip(progress_tx))]
pub async fn search(
    query: &SearchQuery,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<()> {
    let start_time = Instant::now();

    info!("Doing search");

    let progress_tx = &progress_tx;
    let send_engine_progress_update = |engine: Engine, update: EngineProgressUpdate| {
        let _ = progress_tx.send(ProgressUpdate::new(
            ProgressUpdateData::Engine { engine, update },
            start_time,
        ));
    };

    match query.tab {
        SearchTab::All => {
            make_requests(query, progress_tx, start_time, &send_engine_progress_update).await?
        }
        SearchTab::Images if query.config.image_search.enabled => {
            make_image_requests(query, progress_tx, start_time, &send_engine_progress_update)
                .await?
        }
        _ => {
            bail!("unknown tab");
        }
    }

    Ok(())
}

pub async fn autocomplete(config: &Config, query: &str) -> eyre::Result<Vec<String>> {
    let mut requests = Vec::new();
    for &engine in Engine::all() {
        if !config.ui.show_autocomplete {
            break;
        }

        let config = config.engines.get(engine);
        if !config.enabled {
            continue;
        }

        if let Some(request) = engine.request_autocomplete(query) {
            requests.push(async move {
                let response = match request {
                    RequestAutocompleteResponse::Http(request) => {
                        let res = request.send().await?;
                        let body = res.text().await?;
                        engine.parse_autocomplete_response(&body)?
                    }
                    RequestAutocompleteResponse::Instant(response) => response,
                };
                Ok((engine, response))
            });
        }
    }

    let mut autocomplete_futures = Vec::new();
    for request in requests {
        autocomplete_futures.push(request);
    }

    let autocomplete_results_result: eyre::Result<HashMap<_, _>> =
        join_all(autocomplete_futures).await.into_iter().collect();
    let autocomplete_results = autocomplete_results_result?;

    Ok(ranking::merge_autocomplete_responses(
        config,
        autocomplete_results,
    ))
}

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .local_address(IpAddr::from_str("0.0.0.0").unwrap())
        // we pretend to be a normal browser so websites don't block us
        // (since we're not entirely a bot, we're acting on behalf of the user)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0")
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
            headers
        })
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
});

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub search_results: Vec<SearchResult<EngineSearchResult>>,
    pub featured_snippet: Option<FeaturedSnippet>,
    pub answer: Option<Answer>,
    pub infobox: Option<Infobox>,
    #[serde(skip)]
    pub config: Arc<Config>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagesResponse {
    pub image_results: Vec<SearchResult<EngineImageResult>>,
    #[serde(skip)]
    pub config: Arc<Config>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseForTab {
    All(Response),
    Images(ImagesResponse),
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult<R: Serialize> {
    pub result: R,
    pub engines: BTreeSet<Engine>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeaturedSnippet {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engine: Engine,
}

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    #[serde(serialize_with = "serialize_markup")]
    pub html: PreEscaped<String>,
    pub engine: Engine,
}

#[derive(Debug, Clone, Serialize)]
pub struct Infobox {
    #[serde(serialize_with = "serialize_markup")]
    pub html: PreEscaped<String>,
    pub engine: Engine,
}

pub struct AutocompleteResult {
    pub query: String,
    pub score: f64,
}

fn serialize_markup<S>(markup: &PreEscaped<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&markup.0)
}
