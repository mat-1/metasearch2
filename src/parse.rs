//! Helper functions for parsing search engine responses.

use crate::{
    engines::{EngineFeaturedSnippet, EngineResponse, EngineSearchResult},
    normalize::normalize_url,
};

use scraper::{Html, Selector};

#[derive(Default)]
pub struct ParseOpts {
    result: &'static str,
    title: QueryMethod,
    href: QueryMethod,
    description: QueryMethod,

    featured_snippet: &'static str,
    featured_snippet_title: QueryMethod,
    featured_snippet_href: QueryMethod,
    featured_snippet_description: QueryMethod,
}

impl ParseOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn result(mut self, result: &'static str) -> Self {
        self.result = result;
        self
    }

    pub fn title(mut self, title: impl Into<QueryMethod>) -> Self {
        self.title = title.into();
        self
    }

    pub fn href(mut self, href: impl Into<QueryMethod>) -> Self {
        self.href = href.into();
        self
    }

    pub fn description(mut self, description: impl Into<QueryMethod>) -> Self {
        self.description = description.into();
        self
    }

    pub fn featured_snippet(mut self, featured_snippet: &'static str) -> Self {
        self.featured_snippet = featured_snippet;
        self
    }

    pub fn featured_snippet_title(
        mut self,
        featured_snippet_title: impl Into<QueryMethod>,
    ) -> Self {
        self.featured_snippet_title = featured_snippet_title.into();
        self
    }

    pub fn featured_snippet_href(mut self, featured_snippet_href: impl Into<QueryMethod>) -> Self {
        self.featured_snippet_href = featured_snippet_href.into();
        self
    }

    pub fn featured_snippet_description(
        mut self,
        featured_snippet_description: impl Into<QueryMethod>,
    ) -> Self {
        self.featured_snippet_description = featured_snippet_description.into();
        self
    }
}

#[derive(Default)]
pub enum QueryMethod {
    #[default]
    None,
    CssSelector(&'static str),
    Manual(Box<dyn Fn(&scraper::ElementRef) -> eyre::Result<String>>),
}

impl From<&'static str> for QueryMethod {
    fn from(s: &'static str) -> Self {
        QueryMethod::CssSelector(s)
    }
}

impl QueryMethod {
    pub fn call_with_css_selector_override(
        &self,
        el: &scraper::ElementRef,
        with_css_selector: impl Fn(&scraper::ElementRef, &'static str) -> Option<String>,
    ) -> eyre::Result<String> {
        match self {
            QueryMethod::None => Ok(String::new()),
            QueryMethod::CssSelector(s) => Ok(with_css_selector(el, s).unwrap_or_default()),
            QueryMethod::Manual(f) => f(el),
        }
    }

    pub fn call(&self, el: &scraper::ElementRef) -> eyre::Result<String> {
        self.call_with_css_selector_override(el, |el, s| {
            el.select(&Selector::parse(s).unwrap())
                .next()
                .map(|n| n.text().collect::<String>())
        })
    }
}

pub(super) fn parse_html_response_with_opts(
    body: &str,
    opts: ParseOpts,
) -> eyre::Result<EngineResponse> {
    let dom = Html::parse_document(body);

    let mut search_results = Vec::new();

    let ParseOpts {
        result: result_item_query,
        title: title_query_method,
        href: href_query_method,
        description: description_query_method,
        featured_snippet: featured_snippet_query,
        featured_snippet_title: featured_snippet_title_query_method,
        featured_snippet_href: featured_snippet_href_query_method,
        featured_snippet_description: featured_snippet_description_query_method,
    } = opts;

    let result_item_query = Selector::parse(result_item_query).unwrap();

    let results = dom.select(&result_item_query);

    for result in results {
        let title = title_query_method.call(&result)?;
        let url = href_query_method.call_with_css_selector_override(&result, |el, s| {
            el.select(&Selector::parse(s).unwrap()).next().map(|n| {
                n.value()
                    .attr("href")
                    .map(str::to_string)
                    .unwrap_or_else(|| n.text().collect::<String>())
            })
        })?;
        let url = normalize_url(&url)?;
        let description = description_query_method.call(&result)?;

        search_results.push(EngineSearchResult {
            url,
            title,
            description,
        });
    }

    let featured_snippet = if !featured_snippet_query.is_empty() {
        if let Some(featured_snippet) = dom
            .select(&Selector::parse(featured_snippet_query).unwrap())
            .next()
        {
            let title = featured_snippet_title_query_method.call(&featured_snippet)?;
            let url = featured_snippet_href_query_method.call(&featured_snippet)?;
            let url = normalize_url(&url)?;
            let description = featured_snippet_description_query_method.call(&featured_snippet)?;

            // this can happen on google if you search "what's my user agent"
            let is_empty = description.is_empty() && title.is_empty() && url.is_empty();
            if is_empty {
                None
            } else {
                Some(EngineFeaturedSnippet {
                    url,
                    title,
                    description,
                })
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(EngineResponse {
        search_results,
        featured_snippet,
        // this field is used by instant answers, not normal search engines
        answer_html: None,
    })
}
