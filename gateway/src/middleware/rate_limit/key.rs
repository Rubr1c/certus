use std::{borrow::Cow, net::IpAddr};

use crate::{
    config::{RateLimitConfig, RateLimitKey},
    middleware::request_context::RequestContext,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum TokenBucketKey<'a> {
    Ip(IpAddr),
    Token(Cow<'a, str>),
    Header(Cow<'a, str>, axum::http::HeaderValue),
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

#[inline]
pub fn redis_key(key: &TokenBucketKey<'_>) -> String {
    match key {
        TokenBucketKey::Ip(ip) => format!("rate_limit:ip:{}", ip),
        TokenBucketKey::Token(token) => format!("rate_limit:token:{}", token),
        TokenBucketKey::Header(header, value) => {
            format!("rate_limit:header:{}:{}", header, value.to_str().unwrap())
        }
    }
}

#[inline]
pub fn build<'a>(
    config: &'a RateLimitConfig,
    ctx: &'a RequestContext,
    headers: &'a hyper::HeaderMap,
) -> TokenBucketKey<'a> {
    match &config.key {
        RateLimitKey::Ip => TokenBucketKey::Ip(ctx.ip),
        RateLimitKey::Token => match &ctx.token {
            Some(token) => TokenBucketKey::Token(Cow::Borrowed(token.as_str())),
            _ => TokenBucketKey::Ip(ctx.ip),
        },
        RateLimitKey::Header(header) => match headers.get(header.as_str()) {
            Some(val) => TokenBucketKey::Header(
                Cow::Borrowed(header.as_str()),
                val.clone(),
            ),
            _ => TokenBucketKey::Ip(ctx.ip),
        },
    }
}
