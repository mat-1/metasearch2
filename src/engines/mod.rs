use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    sync::LazyLock,
    time::Instant,
};

use futures::future::join_all;
use tokio::sync::mpsc;

use self::search::{bing, brave, google};

pub mod search;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Engine {
    Google,
    Bing,
    Brave,
}

impl Engine {
    pub fn all() -> &'static [Engine] {
        &[Engine::Google, Engine::Bing, Engine::Brave]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Engine::Google => "google",
            Engine::Bing => "bing",
            Engine::Brave => "brave",
        }
    }

    pub fn request(&self, client: &reqwest::Client, query: &str) -> reqwest::RequestBuilder {
        match self {
            Engine::Google => google::request(client, query),
            Engine::Bing => bing::request(client, query),
            Engine::Brave => brave::request(client, query),
        }
    }

    pub fn parse_response(&self, body: &str) -> eyre::Result<EngineResponse> {
        match self {
            Engine::Google => google::parse_response(body),
            Engine::Bing => bing::parse_response(body),
            Engine::Brave => brave::parse_response(body),
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Engine::Google => 1.05,
            Engine::Bing => 1.,
            Engine::Brave => 1.25,
        }
    }
}

#[derive(Debug)]
pub struct EngineSearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

pub struct EngineResponse {
    pub search_results: Vec<EngineSearchResult>,
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
    pub time: f64,
}

impl ProgressUpdate {
    pub fn new(kind: ProgressUpdateKind, engine: Engine, start_time: Instant) -> Self {
        Self {
            kind,
            engine,
            time: start_time.elapsed().as_secs_f64(),
        }
    }
}

impl fmt::Display for ProgressUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ProgressUpdateKind::Requesting => "Requesting",
            ProgressUpdateKind::Downloading => "Downloading",
            ProgressUpdateKind::Parsing => "Parsing",
            ProgressUpdateKind::Done => "Done",
        };

        write!(
            f,
            "{time:.3}s {message} {engine}",
            time = self.time,
            message = message,
            engine = self.engine.name()
        )
    }
}

pub async fn search_with_client_and_engines(
    client: &reqwest::Client,
    engines: &[Engine],
    query: &str,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<Response> {
    let start_time = Instant::now();

    let mut requests = Vec::new();
    for engine in engines {
        requests.push(async {
            let engine = *engine;
            progress_tx.send(ProgressUpdate::new(
                ProgressUpdateKind::Requesting,
                engine,
                start_time,
            ))?;

            let res = engine.request(client, query).send().await?;

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

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

pub async fn search(
    query: &str,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<Response> {
    let engines = Engine::all();
    search_with_client_and_engines(&CLIENT, &engines, query, progress_tx).await
}

#[derive(Debug)]
pub struct Response {
    pub search_results: Vec<SearchResult>,
}
#[derive(Debug)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engines: BTreeSet<Engine>,
    pub score: f64,
}

fn merge_engine_responses(responses: HashMap<Engine, EngineResponse>) -> Response {
    let mut search_results: Vec<SearchResult> = Vec::new();
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
    }

    search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Response { search_results }
}
