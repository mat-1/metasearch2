pub mod calc;
pub mod dictionary;
pub mod ip;
pub mod thesaurus;
pub mod timezone;
pub mod useragent;
pub mod wikipedia;
pub mod rottentomatoes;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}
pub(crate) use regex;
