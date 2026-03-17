use std::borrow::Cow;

use serde::Serialize;

use crate::middleware::request_context::RequestContext;

/// Composite key for dynamic cache entries.
/// Different auth tokens get separate cache entries for the same path.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub struct CacheKey<'a> {
    pub token: Option<Cow<'a, str>>,
    pub path: Cow<'a, str>,
}

pub type OwnedCacheKey = CacheKey<'static>;

impl CacheKey<'_> {
    pub fn into_owned(self) -> OwnedCacheKey {
        CacheKey {
            token: self.token.map(|t| Cow::Owned(t.into_owned())),
            path: Cow::Owned(self.path.into_owned()),
        }
    }
}

#[inline]
pub fn build<'a>(ctx: &'a RequestContext) -> CacheKey<'a> {
    CacheKey {
        token: ctx.token.as_deref().map(Cow::Borrowed),
        path: ctx.query.as_ref().map_or_else(
            || Cow::Borrowed(ctx.path.as_ref()),
            |q| Cow::Owned(format!("{}?{}", ctx.path, q)),
        ),
    }
}
