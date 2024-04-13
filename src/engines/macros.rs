#[macro_export]
macro_rules! engines {
    ($($engine:ident = $id:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Engine {
            $($engine,)*
        }

        impl Engine {
            #[must_use]
            pub fn all() -> &'static [Engine] {
                &[$(Engine::$engine,)*]
            }

            #[must_use]
            pub fn id(&self) -> &'static str {
                match self {
                    $(Engine::$engine => $id,)*
                }
            }
        }

        impl FromStr for Engine {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($id => Ok(Engine::$engine),)*
                    _ => Err(()),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_parse_response {
    ($res:ident, $module:ident::$engine_id:ident::None) => {
        None
    };
    ($res:ident, $module:ident::$engine_id:ident::$parse_response:ident) => {
        Some($module::$engine_id::$parse_response($res.into()))
    };
}

#[macro_export]
macro_rules! engine_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn request(&self, query: &SearchQuery) -> RequestResponse {
                #[allow(clippy::useless_conversion)]
                match self {
                    $(
                        Engine::$engine => $module::$engine_id::$request(query).into(),
                    )*
                    _ => RequestResponse::None,
                }
            }

            pub fn parse_response(&self, res: &HttpResponse) -> eyre::Result<EngineResponse> {
                #[allow(clippy::useless_conversion)]
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { res, $module::$engine_id::$parse_response }
                            .ok_or_else(|| eyre::eyre!("engine {self:?} can't parse response"))?,
                    )*
                    _ => eyre::bail!("engine {self:?} can't parse response"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_autocomplete_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn request_autocomplete(&self, query: &str) -> Option<RequestAutocompleteResponse> {
                match self {
                    $(
                        Engine::$engine => Some($module::$engine_id::$request(query).into()),
                    )*
                    _ => None,
                }
            }

            pub fn parse_autocomplete_response(&self, body: &str) -> eyre::Result<Vec<String>> {
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { body, $module::$engine_id::$parse_response }
                            .ok_or_else(|| eyre::eyre!("engine {self:?} can't parse autocomplete response"))?,
                    )*
                    _ => eyre::bail!("engine {self:?} can't parse autocomplete response"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_postsearch_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn postsearch_request(&self, response: &Response) -> Option<reqwest::RequestBuilder> {
                match self {
                    $(
                        Engine::$engine => $module::$engine_id::$request(response),
                    )*
                    _ => None,
                }
            }

            #[must_use]
            pub fn postsearch_parse_response(&self, res: &HttpResponse) -> Option<String> {
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { res, $module::$engine_id::$parse_response }?,
                    )*
                    _ => None,
                }
            }
        }
    };
}
