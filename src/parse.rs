//! Helper functions for parsing search engine responses.

use crate::{
    engines::{EngineResponse, EngineSearchResult},
    normalize::normalize_url,
};

use scraper::{Html, Selector};

pub struct ParseOpts<A, B, C>
where
    A: Into<QueryMethod>,
    B: Into<QueryMethod>,
    C: Into<QueryMethod>,
{
    pub result_item: &'static str,
    pub title: A,
    pub href: B,
    pub description: C,
}

pub enum QueryMethod {
    CssSelector(&'static str),
    Manual(Box<dyn Fn(&scraper::ElementRef) -> eyre::Result<String>>),
}

impl From<&'static str> for QueryMethod {
    fn from(s: &'static str) -> Self {
        QueryMethod::CssSelector(s)
    }
}

pub(super) fn parse_html_response_with_opts<A, B, C>(
    body: &str,
    opts: ParseOpts<A, B, C>,
) -> eyre::Result<EngineResponse>
where
    A: Into<QueryMethod>,
    B: Into<QueryMethod>,
    C: Into<QueryMethod>,
{
    let dom = Html::parse_document(body);

    let mut search_results = Vec::new();

    let ParseOpts {
        result_item: result_item_query,
        title: title_query_method,
        href: href_query_method,
        description: description_query_method,
    } = opts;
    let title_query_method = title_query_method.into();
    let href_query_method = href_query_method.into();
    let description_query_method = description_query_method.into();

    let result_item_query = Selector::parse(result_item_query).unwrap();

    let result_items = dom.select(&result_item_query);

    for result_item in result_items {
        let title = match title_query_method {
            QueryMethod::CssSelector(s) => result_item
                .select(&Selector::parse(s).unwrap())
                .next()
                .map(|n| n.text().collect::<String>())
                .unwrap_or_default(),
            QueryMethod::Manual(ref f) => f(&result_item)?,
        };

        let url = match href_query_method {
            QueryMethod::CssSelector(s) => result_item
                .select(&Selector::parse(s).unwrap())
                .next()
                .map(|n| {
                    n.value()
                        .attr("href")
                        .map(str::to_string)
                        .unwrap_or_else(|| n.text().collect::<String>())
                })
                .unwrap_or_default(),
            QueryMethod::Manual(ref f) => f(&result_item)?,
        };
        let url = normalize_url(&url)?;

        let description = match description_query_method {
            QueryMethod::CssSelector(s) => result_item
                .select(&Selector::parse(s).unwrap())
                .next()
                .map(|n| n.text().collect::<String>())
                .unwrap_or_default(),
            QueryMethod::Manual(ref f) => f(&result_item)?,
        };

        search_results.push(EngineSearchResult {
            url,
            title,
            description,
        });
    }

    Ok(EngineResponse { search_results })
}
