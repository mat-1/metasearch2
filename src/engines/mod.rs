use std::{
    collections::{BTreeSet, HashMap},
    net::IpAddr,
    ops::Deref,
    str::FromStr,
    sync::LazyLock,
    time::Instant,
};

use futures::future::join_all;
use tokio::sync::mpsc;

pub mod answer;
pub mod search;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Engine {
    // search
    Google,
    Bing,
    Brave,
    // answer
    Useragent,
    Ip,
    Calc,
}

impl Engine {
    pub fn all() -> &'static [Engine] {
        &[
            Engine::Google,
            Engine::Bing,
            Engine::Brave,
            Engine::Useragent,
            Engine::Ip,
            Engine::Calc,
        ]
    }

    pub fn id(&self) -> &'static str {
        match self {
            Engine::Google => "google",
            Engine::Bing => "bing",
            Engine::Brave => "brave",
            Engine::Useragent => "useragent",
            Engine::Ip => "ip",
            Engine::Calc => "calc",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Engine::Google => 1.05,
            Engine::Bing => 1.,
            Engine::Brave => 1.25,
            _ => 1.,
        }
    }

    pub fn request(&self, query: &SearchQuery) -> RequestResponse {
        match self {
            Engine::Google => search::google::request(query).into(),
            Engine::Bing => search::bing::request(query).into(),
            Engine::Brave => search::brave::request(query).into(),
            Engine::Useragent => answer::useragent::request(query).into(),
            Engine::Ip => answer::ip::request(query).into(),
            Engine::Calc => answer::calc::request(query).into(),
        }
    }

    pub fn parse_response(&self, body: &str) -> eyre::Result<EngineResponse> {
        match self {
            Engine::Google => search::google::parse_response(body),
            Engine::Bing => search::bing::parse_response(body),
            Engine::Brave => search::brave::parse_response(body),
            _ => eyre::bail!("engine {self:?} can't parse response"),
        }
    }

    pub fn request_autocomplete(&self, query: &str) -> Option<RequestAutocompleteResponse> {
        match self {
            Engine::Google => Some(search::google::request_autocomplete(query).into()),
            Engine::Calc => Some(answer::calc::request_autocomplete(query).into()),
            _ => None,
        }
    }

    pub fn parse_autocomplete_response(&self, body: &str) -> eyre::Result<Vec<String>> {
        match self {
            Engine::Google => search::google::parse_autocomplete_response(body),
            _ => eyre::bail!("engine {self:?} can't parse autocomplete response"),
        }
    }
}

pub struct SearchQuery {
    pub query: String,
    pub request_headers: HashMap<String, String>,
    pub ip: String,
}

impl Deref for SearchQuery {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

pub enum RequestResponse {
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
    Http(reqwest::RequestBuilder),
    Instant(Vec<String>),
}
impl From<reqwest::RequestBuilder> for RequestAutocompleteResponse {
    fn from(req: reqwest::RequestBuilder) -> Self {
        Self::Http(req)
    }
}
impl From<Vec<String>> for RequestAutocompleteResponse {
    fn from(res: Vec<String>) -> Self {
        Self::Instant(res)
    }
}

#[derive(Debug)]
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
    pub answer_html: Option<String>,
}

impl EngineResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answer_html(html: String) -> Self {
        Self {
            answer_html: Some(html),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub enum ProgressUpdateKind {
    Requesting,
    Downloading,
    Parsing,
    Done,
}

#[derive(Debug)]
pub struct ProgressUpdate {
    pub kind: ProgressUpdateKind,
    pub engine: Engine,
    pub time: u64,
}

impl ProgressUpdate {
    pub fn new(kind: ProgressUpdateKind, engine: Engine, start_time: Instant) -> Self {
        Self {
            kind,
            engine,
            time: start_time.elapsed().as_millis() as u64,
        }
    }
}

pub async fn search_with_engines(
    engines: &[Engine],
    query: &SearchQuery,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<Response> {
    let start_time = Instant::now();

    let mut requests = Vec::new();
    for engine in engines {
        requests.push(async {
            let engine = *engine;

            let request_response = engine.request(query).into();

            let response = match request_response {
                RequestResponse::Http(request) => {
                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateKind::Requesting,
                        engine,
                        start_time,
                    ))?;

                    let res = request.send().await?;

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateKind::Downloading,
                        engine,
                        start_time,
                    ))?;

                    let body = res.text().await?;

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateKind::Parsing,
                        engine,
                        start_time,
                    ))?;

                    let response = engine.parse_response(&body)?;

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateKind::Done,
                        engine,
                        start_time,
                    ))?;

                    response
                }
                RequestResponse::Instant(response) => response,
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

    Ok(merge_engine_responses(responses))
}

pub async fn autocomplete_with_engines(
    engines: &[Engine],
    query: &str,
) -> eyre::Result<Vec<String>> {
    let mut requests = Vec::new();
    for engine in engines {
        if let Some(request) = engine.request_autocomplete(query) {
            requests.push(async {
                let response = match request {
                    RequestAutocompleteResponse::Http(request) => {
                        let res = request.send().await?;
                        let body = res.text().await?;
                        engine.parse_autocomplete_response(&body)?
                    }
                    RequestAutocompleteResponse::Instant(response) => response,
                };
                Ok((*engine, response))
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

    Ok(merge_autocomplete_responses(autocomplete_results))
}

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .local_address(IpAddr::from_str("0.0.0.0").unwrap())
        .build()
        .unwrap()
});

pub async fn search(
    query: SearchQuery,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<Response> {
    let engines = Engine::all();
    search_with_engines(&engines, &query, progress_tx).await
}

pub async fn autocomplete(query: &str) -> eyre::Result<Vec<String>> {
    let engines = Engine::all();
    autocomplete_with_engines(&engines, query).await
}

#[derive(Debug)]
pub struct Response {
    pub search_results: Vec<SearchResult>,
    pub featured_snippet: Option<FeaturedSnippet>,
    pub answer: Option<Answer>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engines: BTreeSet<Engine>,
    pub score: f64,
}

#[derive(Debug)]
pub struct FeaturedSnippet {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engine: Engine,
}

#[derive(Debug)]
pub struct Answer {
    pub html: String,
    pub engine: Engine,
}

fn merge_engine_responses(responses: HashMap<Engine, EngineResponse>) -> Response {
    let mut search_results: Vec<SearchResult> = Vec::new();
    let mut featured_snippet: Option<FeaturedSnippet> = None;
    let mut answer: Option<Answer> = None;

    for (engine, response) in responses {
        for (result_index, search_result) in response.search_results.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine.weight();

            if let Some(existing_result) = search_results
                .iter_mut()
                .find(|r| r.url == search_result.url)
            {
                // if the weight of this engine is higher than every other one then replace the title and description
                if engine.weight()
                    > existing_result
                        .engines
                        .iter()
                        .map(Engine::weight)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.)
                {
                    existing_result.title = search_result.title;
                    existing_result.description = search_result.description;
                }

                existing_result.engines.insert(engine);
                existing_result.score += result_score;
            } else {
                search_results.push(SearchResult {
                    url: search_result.url,
                    title: search_result.title,
                    description: search_result.description,
                    engines: [engine].iter().cloned().collect(),
                    score: result_score,
                });
            }
        }

        if let Some(engine_featured_snippet) = response.featured_snippet {
            // if it has a higher weight than the current featured snippet
            let featured_snippet_weight = featured_snippet
                .as_ref()
                .map(|s| s.engine.weight())
                .unwrap_or(0.);
            if engine.weight() > featured_snippet_weight {
                featured_snippet = Some(FeaturedSnippet {
                    url: engine_featured_snippet.url,
                    title: engine_featured_snippet.title,
                    description: engine_featured_snippet.description,
                    engine,
                });
            }
        }

        if let Some(engine_answer_html) = response.answer_html {
            // if it has a higher weight than the current answer
            let answer_weight = answer.as_ref().map(|s| s.engine.weight()).unwrap_or(0.);
            if engine.weight() > answer_weight {
                answer = Some(Answer {
                    html: engine_answer_html,
                    engine,
                });
            }
        }
    }

    search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Response {
        search_results,
        featured_snippet,
        answer,
    }
}

pub struct AutocompleteResult {
    pub query: String,
    pub score: f64,
}

fn merge_autocomplete_responses(responses: HashMap<Engine, Vec<String>>) -> Vec<String> {
    let mut autocomplete_results: Vec<AutocompleteResult> = Vec::new();

    for (engine, response) in responses {
        for (result_index, autocomplete_result) in response.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine.weight();

            if let Some(existing_result) = autocomplete_results
                .iter_mut()
                .find(|r| r.query == autocomplete_result)
            {
                existing_result.score += result_score;
            } else {
                autocomplete_results.push(AutocompleteResult {
                    query: autocomplete_result,
                    score: result_score,
                });
            }
        }
    }

    autocomplete_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    autocomplete_results.into_iter().map(|r| r.query).collect()
}
