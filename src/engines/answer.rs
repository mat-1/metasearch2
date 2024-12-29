pub mod colorpicker;
pub mod dictionary;
pub mod fend;
pub mod ip;
pub mod notepad;
pub mod numbat;
pub mod thesaurus;
pub mod timezone;
pub mod useragent;
pub mod wikipedia;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($re).unwrap());
        &RE
    }};
}
pub(crate) use regex;
