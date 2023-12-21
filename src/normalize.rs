use url::Url;

pub fn normalize_url(url: &str) -> eyre::Result<String> {
    let url = url.trim_end_matches('#');
    if url.is_empty() {
        return Ok(String::new());
    }

    let Ok(mut url) = Url::parse(url) else {
        eprintln!("failed to parse url: {url}");
        return Ok(url.to_string());
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

    // convert minecraft.fandom.com/wiki/ to minecraft.wiki/w/
    if url.host_str() == Some("minecraft.fandom.com") {
        let path = url.path().to_string();
        if let Some(path) = path.strip_prefix("/wiki/") {
            url.set_host(Some("minecraft.wiki")).unwrap();
            url.set_path(&format!("/w/{path}"));
        }
    }

    // url decode and encode path
    let path = url.path().to_string();
    let path = urlencoding::decode(&path)?;
    url.set_path(path.as_ref());

    let url = url.to_string();
    // remove trailing slash
    let url = if let Some(url) = url.strip_suffix('/') {
        url.to_string()
    } else {
        url
    };

    Ok(url)
}
