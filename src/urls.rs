use std::borrow::Cow;

use tracing::{error, warn};
use url::Url;

use crate::config::UrlsConfig;

#[tracing::instrument]
pub fn normalize_url(url: &str) -> String {
    let url = url.trim_end_matches('#');
    if url.is_empty() {
        warn!("url is empty");
        return String::new();
    }

    let Ok(mut url) = Url::parse(url) else {
        error!("failed to parse url");
        return url.to_string();
    };

    // make sure the scheme is https
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap();
    }

    // remove fragment
    url.set_fragment(None);

    // remove trailing slash
    let path = url.path().to_string();
    if let Some(path) = path.strip_suffix('/') {
        url.set_path(path);
    }

    // remove tracking params
    let query_pairs = url.query_pairs().into_owned();
    let mut new_query_pairs = Vec::new();
    const TRACKING_PARAMS: &[&str] = &["ref_src", "_sm_au_"];
    for (key, value) in query_pairs {
        if !TRACKING_PARAMS.contains(&key.as_str()) {
            new_query_pairs.push((key, value));
        }
    }
    if new_query_pairs.is_empty() {
        url.set_query(None);
    } else {
        url.set_query(Some(
            &url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(new_query_pairs)
                .finish(),
        ));
    }

    // url decode and encode path
    let path = url.path().to_string();
    let path = match urlencoding::decode(&path) {
        Ok(path) => path,
        Err(e) => {
            warn!("failed to decode path: {e}");
            Cow::Owned(path)
        }
    };
    url.set_path(path.as_ref());

    let url = url.to_string();
    // remove trailing slash
    let url = if let Some(url) = url.strip_suffix('/') {
        url.to_string()
    } else {
        url
    };

    url
}

pub fn apply_url_replacements(url: &str, urls_config: &UrlsConfig) -> String {
    let Ok(mut url) = Url::parse(url) else {
        error!("failed to parse url");
        return url.to_string();
    };

    let host = url.host_str().unwrap_or_default().to_owned();

    let path = url.path();
    let path = path.strip_prefix("/").unwrap_or(path).to_owned();
    for (replace_from, replace_to) in &urls_config.replace {
        let new_host = if replace_from.domain.starts_with(".") {
            if host.ends_with(&replace_from.domain) {
                if replace_to.domain.starts_with(".") {
                    let host_without_suffix = host
                        .strip_suffix(&replace_from.domain)
                        .expect("host was already verified to end in replace_from.domain");
                    format!("{}{}", host_without_suffix, replace_to.domain)
                } else {
                    replace_to.domain.clone()
                }
            } else {
                continue;
            }
        } else if host == replace_from.domain {
            replace_to.domain.clone()
        } else {
            continue;
        };

        // host matches, now check path

        let new_path = if let Some(path) = path.strip_prefix(&replace_from.path) {
            format!("{}{path}", replace_to.path)
        } else {
            continue;
        };

        let _ = url.set_host(Some(&new_host));
        url.set_path(&new_path);
    }

    normalize_url(&url.to_string())
}
pub fn get_url_weight(url: &str, urls_config: &UrlsConfig) -> f64 {
    let Ok(url) = Url::parse(url) else {
        error!("failed to parse url");
        return 1.;
    };

    let host = url.host_str().unwrap_or_default().to_owned();
    let path = url.path().strip_prefix("/").unwrap_or_default().to_owned();
    for (check, weight) in &urls_config.weight {
        if check.domain.starts_with(".") {
            if host.ends_with(&check.domain) && path.starts_with(&check.path) {
                return *weight;
            }
        } else if host == check.domain && path.starts_with(&check.path) {
            return *weight;
        }
    }

    1.
}

#[cfg(test)]
mod tests {
    use crate::config::DomainAndPath;

    use super::*;

    fn test_replacement(from: &str, to: &str, url: &str, expected: &str) {
        let urls_config = UrlsConfig {
            replace: vec![(DomainAndPath::from_str(from), DomainAndPath::from_str(to))],
            weight: vec![],
        };
        let normalized_url = apply_url_replacements(url, &urls_config);
        assert_eq!(normalized_url, expected);
    }

    #[test]
    fn test_replace_url() {
        test_replacement(
            "minecraft.fandom.com/wiki/",
            "minecraft.wiki/w/",
            "https://minecraft.fandom.com/wiki/Java_Edition",
            "https://minecraft.wiki/w/Java_Edition",
        );
    }
    #[test]
    fn test_replace_wildcard_domain_with_absolute() {
        test_replacement(
            ".medium.com",
            "scribe.rip",
            "https://example.medium.com/asdf",
            "https://scribe.rip/asdf",
        );
    }
    #[test]
    fn test_replace_wildcard_domain_with_wildcard() {
        test_replacement(
            ".medium.com",
            ".scribe.rip",
            "https://example.medium.com/asdf",
            "https://example.scribe.rip/asdf",
        );
    }
    #[test]
    fn test_non_matching_wildcard() {
        test_replacement(
            ".medium.com",
            ".scribe.rip",
            "https://medium.com/asdf",
            "https://medium.com/asdf",
        );
    }
}
