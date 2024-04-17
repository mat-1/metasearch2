//! These search engines are requested after we've built the main search
//! results. They can only show stuff in infoboxes and don't get requested if
//! an infobox was added by another earlier engine.

pub mod docs_rs;
pub mod github;
pub mod mdn;
pub mod minecraft_wiki;
pub mod stackexchange;
