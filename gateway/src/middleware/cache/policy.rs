use hyper::header;

use crate::config::RouteConfig;

pub struct CachePolicy {
    pub no_store: bool,
    pub max_age: Option<u64>,
    pub lookup: bool,
    // emit bypass event
    pub bypass: bool,
}

#[inline]
pub fn build(
    route: &RouteConfig,
    method: &hyper::Method,
    token: Option<&str>,
    headers: &hyper::HeaderMap,
) -> CachePolicy {
    let cacheable_method = *method == hyper::Method::GET;
    let mut no_store = route.no_cache || !cacheable_method;

    if route.no_cache || !cacheable_method {
        return CachePolicy {
            no_store,
            max_age: None,
            lookup: false,
            bypass: true,
        };
    }

    let mut h_no_cache = false;
    let mut h_no_store = false;
    let mut h_private = false;
    let mut h_public = false;
    let mut h_max_age = None;
    let mut h_s_max_age = None;

    if let Some(cc_header) =
        headers.get(header::CACHE_CONTROL).and_then(|h| h.to_str().ok())
    {
        for part in cc_header.split(',') {
            let part = part.trim();
            match part {
                "no-cache" => h_no_cache = true,
                "no-store" => h_no_store = true,
                "private" => h_private = true,
                "public" => h_public = true,
                _ if part.starts_with("max-age=") => {
                    h_max_age = part[8..].parse::<u64>().ok();
                }
                _ if part.starts_with("s-maxage=") => {
                    h_s_max_age = part[9..].parse::<u64>().ok();
                }
                _ => {}
            }
        }
    }

    if h_s_max_age.is_some() {
        h_max_age = h_s_max_age;
    }

    no_store = h_no_store || h_private || token.is_some() && !h_public;

    CachePolicy {
        no_store,
        max_age: h_max_age,
        lookup: !(no_store || h_no_cache),
        bypass: false,
    }
}
