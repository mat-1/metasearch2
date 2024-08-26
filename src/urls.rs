use std::borrow::Cow;

use tracing::{error, warn};
use url::Url;

use crate::config::{HostAndPath, UrlsConfig};

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

impl HostAndPath {
    pub fn contains(&self, host: &str, path: &str) -> bool {
        if self.host.starts_with('.') {
            if !host.ends_with(&self.host) {
                return false;
            }
        } else if host != self.host {
            return false;
        }

        if self.path.ends_with('/') || self.path.is_empty() {
            path.starts_with(&self.path)
        } else {
            path == self.path
        }
    }

    pub fn replace(
        replace_from: &HostAndPath,
        replace_with: &HostAndPath,
        real_url: &HostAndPath,
    ) -> Option<(String, String)> {
        let new_host = if replace_from.host.starts_with(".") {
            if replace_with.host.starts_with(".") {
                if let Some(host_without_suffix) = real_url.host.strip_suffix(&replace_from.host) {
                    format!("{host_without_suffix}{}", replace_with.host)
                } else {
                    return None;
                }
            } else if real_url.host.ends_with(&replace_from.host) {
                replace_with.host.to_owned()
            } else {
                return None;
            }
        } else if real_url.host == replace_from.host {
            replace_with.host.clone()
        } else {
            return None;
        };

        // host matches, now check path

        let new_path = if replace_from.path.ends_with('/') || replace_from.path.is_empty() {
            if replace_with.path.ends_with('/') || replace_with.path.is_empty() {
                if let Some(path_without_prefix) = real_url.path.strip_prefix(&replace_from.path) {
                    format!("{}{path_without_prefix}", replace_with.path)
                } else {
                    return None;
                }
            } else if real_url.path.starts_with(&replace_from.path) {
                replace_with.path.clone()
            } else {
                return None;
            }
        } else if real_url.path == replace_from.path {
            replace_with.path.clone()
        } else {
            return None;
        };

        Some((new_host, new_path))
    }
}

pub fn apply_url_replacements(url: &str, urls_config: &UrlsConfig) -> String {
    let Ok(mut url) = Url::parse(url) else {
        error!("failed to parse url");
        return url.to_string();
    };

    let host = url.host_str().unwrap_or_default().to_owned();

    let path = url
        .path()
        .strip_prefix("/")
        .unwrap_or(url.path())
        .to_owned();
    let real_url = HostAndPath { host, path };
    for (replace_from, replace_to) in &urls_config.replace {
        if let Some((new_host, new_path)) =
            HostAndPath::replace(replace_from, replace_to, &real_url)
        {
            let _ = url.set_host(Some(&new_host));
            url.set_path(&new_path);
            break;
        }
    }

    normalize_url(url.as_ref())
}
pub fn get_url_weight(url: &str, urls_config: &UrlsConfig) -> f64 {
    let Ok(url) = Url::parse(url) else {
        error!("failed to parse url");
        return 1.;
    };

    let host = url.host_str().unwrap_or_default().to_owned();
    let path = url.path().strip_prefix("/").unwrap_or_default().to_owned();
    for (check, weight) in &urls_config.weight {
        if check.contains(&host, &path) {
            return *weight;
        }
    }

    1.
}

#[cfg(test)]
mod tests {
    use crate::config::HostAndPath;

    use super::*;

    fn test_replacement(from: &str, to: &str, url: &str, expected: &str) {
        let urls_config = UrlsConfig {
            replace: vec![(HostAndPath::new(from), HostAndPath::new(to))],
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
    fn test_replace_wildcard_host_with_absolute() {
        test_replacement(
            ".medium.com",
            "scribe.rip",
            "https://example.medium.com/asdf",
            "https://scribe.rip/asdf",
        );
    }
    #[test]
    fn test_replace_wildcard_host_with_wildcard() {
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
    #[test]
    fn test_non_matching_wildcard_to_absolute() {
        test_replacement(
            ".medium.com",
            "scribe.rip",
            "https://example.com/asdf",
            "https://example.com/asdf",
        );
    }
}
