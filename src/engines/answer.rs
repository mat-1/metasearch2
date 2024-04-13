pub mod dictionary;
pub mod fend;
pub mod ip;
pub mod numbat;
pub mod thesaurus;
pub mod timezone;
pub mod useragent;
pub mod wikipedia;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}
pub(crate) use regex;
