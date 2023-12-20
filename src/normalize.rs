use url::Url;

pub fn normalize_url(url: &str) -> eyre::Result<String> {
    if url.is_empty() {
        return Ok(String::new());
    }

    let mut url = Url::parse(url)?;

    // make sure the scheme is https
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap();
    }

    // remove trailing slash
    let path = url.path().to_string();
    if let Some(path) = path.strip_suffix('/') {
        url.set_path(path);
    }

    // remove ref_src tracking param
    let query_pairs = url.query_pairs().into_owned();
    let mut new_query_pairs = Vec::new();
    for (key, value) in query_pairs {
        if key != "ref_src" {
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
    let path = urlencoding::decode(&path)?;
    url.set_path(&path.to_string());

    let url = url.to_string();
    // remove trailing slash
    let url = if let Some(url) = url.strip_suffix('/') {
        url.to_string()
    } else {
        url
    };

    return Ok(url);
}
