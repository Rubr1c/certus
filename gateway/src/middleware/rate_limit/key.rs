use std::{borrow::Cow, net::IpAddr};

use axum::http::HeaderValue;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum TokenBucketKey<'a> {
    Ip(IpAddr),
    Token(Cow<'a, str>),
    Header(Cow<'a, str>, HeaderValue),
}

pub type OwnedTokenBucketKey = TokenBucketKey<'static>;

impl TokenBucketKey<'_> {
    pub fn into_owned(self) -> OwnedTokenBucketKey {
        match self {
            TokenBucketKey::Ip(ip) => TokenBucketKey::Ip(ip),
            TokenBucketKey::Token(token) => {
                TokenBucketKey::Token(Cow::Owned(token.into_owned()))
            }
            TokenBucketKey::Header(header, value) => {
                TokenBucketKey::Header(Cow::Owned(header.into_owned()), value)
            }
        }
    }
}

pub fn redis_key(key: &TokenBucketKey<'_>) -> String {
    match key {
        TokenBucketKey::Ip(ip) => format!("rate_limit:ip:{}", ip),
        TokenBucketKey::Token(token) => format!("rate_limit:token:{}", token),
        TokenBucketKey::Header(header, value) => {
            format!("rate_limit:header:{}:{}", header, value.to_str().unwrap())
        }
    }
}
