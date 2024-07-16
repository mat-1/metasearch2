use std::{collections::HashMap, sync::Arc};

use crate::{
    config::Config,
    urls::{apply_url_replacements, get_url_weight},
};

use super::{
    Answer, AutocompleteResult, Engine, EngineImageResult, EngineImagesResponse, EngineResponse,
    EngineSearchResult, FeaturedSnippet, ImagesResponse, Infobox, Response, SearchResult,
};

pub fn merge_engine_responses(
    config: Arc<Config>,
    responses: HashMap<Engine, EngineResponse>,
) -> Response {
    let mut search_results: Vec<SearchResult<EngineSearchResult>> = Vec::new();
    let mut featured_snippet: Option<FeaturedSnippet> = None;
    let mut answer: Option<Answer> = None;
    let mut infobox: Option<Infobox> = None;

    for (engine, response) in responses {
        let engine_config = config.engines.get(engine);

        for (result_index, mut search_result) in response.search_results.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a
            // score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine_config.weight;

            // apply url config here
            search_result.url = apply_url_replacements(&search_result.url, &config.urls);
            let url_weight = get_url_weight(&search_result.url, &config.urls);
            if url_weight <= 0. {
                continue;
            }
            let result_score = result_score * url_weight;

            if let Some(existing_result) = search_results
                .iter_mut()
                .find(|r| r.result.url == search_result.url)
            {
                // if the weight of this engine is higher than every other one then replace the
                // title and description
                if engine_config.weight
                    > existing_result
                        .engines
                        .iter()
                        .map(|&other_engine| {
                            let other_engine_config = config.engines.get(other_engine);
                            other_engine_config.weight
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.)
                {
                    existing_result.result.title = search_result.title;
                    existing_result.result.description = search_result.description;
                }

                existing_result.engines.insert(engine);
                existing_result.score += result_score;
            } else {
                search_results.push(SearchResult {
                    result: search_result,
                    engines: [engine].iter().copied().collect(),
                    score: result_score,
                });
            }
        }

        if let Some(mut engine_featured_snippet) = response.featured_snippet {
            // if it has a higher weight than the current featured snippet
            let featured_snippet_weight = featured_snippet.as_ref().map_or(0., |s| {
                let other_engine_config = config.engines.get(s.engine);
                other_engine_config.weight
            });

            // url config applies to featured snippets too
            engine_featured_snippet.url =
                apply_url_replacements(&engine_featured_snippet.url, &config.urls);
            let url_weight = get_url_weight(&engine_featured_snippet.url, &config.urls);
            if url_weight <= 0. {
                continue;
            }
            let featured_snippet_weight = featured_snippet_weight * url_weight;

            if engine_config.weight > featured_snippet_weight {
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
            let answer_weight = answer.as_ref().map_or(0., |s| {
                let other_engine_config = config.engines.get(s.engine);
                other_engine_config.weight
            });
            if engine_config.weight > answer_weight {
                answer = Some(Answer {
                    html: engine_answer_html,
                    engine,
                });
            }
        }

        if let Some(engine_infobox_html) = response.infobox_html {
            // if it has a higher weight than the current infobox
            let infobox_weight = infobox.as_ref().map_or(0., |s| {
                let other_engine_config = config.engines.get(s.engine);
                other_engine_config.weight
            });
            if engine_config.weight > infobox_weight {
                infobox = Some(Infobox {
                    html: engine_infobox_html,
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
        infobox,
        config,
    }
}

pub fn merge_autocomplete_responses(
    config: &Config,
    responses: HashMap<Engine, Vec<String>>,
) -> Vec<String> {
    let mut autocomplete_results: Vec<AutocompleteResult> = Vec::new();

    for (engine, response) in responses {
        let engine_config = config.engines.get(engine);

        for (result_index, autocomplete_result) in response.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a
            // score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine_config.weight;

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

pub fn merge_images_responses(
    config: Arc<Config>,
    responses: HashMap<Engine, EngineImagesResponse>,
) -> ImagesResponse {
    let mut image_results: Vec<SearchResult<EngineImageResult>> = Vec::new();

    for (engine, response) in responses {
        let engine_config = config.engines.get(engine);

        for (result_index, image_result) in response.image_results.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a
            // score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine_config.weight;

            if let Some(existing_result) = image_results
                .iter_mut()
                .find(|r| r.result.image_url == image_result.image_url)
            {
                // if the weight of this engine is higher than every other one then replace the
                // title and page url
                if engine_config.weight
                    > existing_result
                        .engines
                        .iter()
                        .map(|&other_engine| {
                            let other_engine_config = config.engines.get(other_engine);
                            other_engine_config.weight
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.)
                {
                    existing_result.result.title = image_result.title;
                    existing_result.result.page_url = image_result.page_url;
                }

                existing_result.engines.insert(engine);
                existing_result.score += result_score;
            } else {
                image_results.push(SearchResult {
                    result: image_result,
                    engines: [engine].iter().copied().collect(),
                    score: result_score,
                });
            }
        }
    }

    image_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    ImagesResponse {
        image_results,
        config,
    }
}
