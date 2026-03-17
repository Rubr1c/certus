use std::net::IpAddr;

use hyper::{HeaderMap, header};

// TODO: this wont work forwarded has many params in it
//       so it needs to be parsed right now it will
//       always fallback to sender_ip.
//       there is also X-Forwarded-For that is legacy but
//       still used.
//       better to make it config based later so parsing
//       is not needed each time if no proxy is involved
//       or user does not care about the ip and does not
//       rate limit with it.
#[inline]
pub fn extract_ip(headers: &HeaderMap, sender_ip: IpAddr) -> IpAddr {
    headers
        .get(header::FORWARDED)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.parse::<IpAddr>().unwrap_or(sender_ip))
        .unwrap_or(sender_ip)
}

// TODO: set real ip in forwarded response header
