use hyper::header;

use crate::{
    config::{AuthType, Config},
    error::GatewayError,
};

use super::jwt;

/// Tries to authenticate the user if route requires auth
///
/// # Arguments
///
/// * `headers` - header ref to inject headers
/// * `token` - optional token if exists
/// * `config` - gateway config
/// * `needs_auth` - whether the route requires authentication
///
/// # Errors
///
/// Returns an error if:
/// * Authentication is enabled and no token was provided
/// * Token failed to be decoded [`decode`]
#[inline(always)]
pub fn run(
    headers: &mut hyper::HeaderMap<axum::http::HeaderValue>,
    token: Option<&str>,
    config: &Config,
    needs_auth: bool,
) -> Result<(), GatewayError> {
    if needs_auth {
        tracing::debug!(
            has_token = token.is_some(),
            auth_method = ?config.auth.method,
            "Authenticating user"
        );
        match token {
            Some(t) => {
                match &config.auth.method {
                    AuthType::JWT { secret, algorithm } => {
                        match jwt::decode(t, secret, algorithm) {
                            Ok(claims) => {
                                //TODO: put claims in header
                                tracing::debug!("User authenticated");

                                if let Some(id) = claims.user_id {
                                    headers.insert(
                                        "X-User-Id",
                                        axum::http::HeaderValue::from_str(
                                            id.as_str(),
                                        )
                                        .map_err(|_| {
                                            GatewayError::InternalServerError
                                        })?,
                                    );
                                }

                                if let Some(role) = claims.role {
                                    headers.insert(
                                        "X-User-Role",
                                        axum::http::HeaderValue::from_str(
                                            role.as_str(),
                                        )
                                        .map_err(|_| {
                                            GatewayError::InternalServerError
                                        })?,
                                    );
                                }

                                return Ok(());
                            }
                            Err(e) => {
                                tracing::debug!(
                                    has_token = true,
                                    auth_method = ?config.auth.method,
                                    "User not authenticated"
                                );
                                return Err(e);
                            }
                        }
                    }
                    AuthType::None => return Ok(()),
                }
            }
            _ => {
                tracing::debug!(
                    has_token = false,
                    auth_method = ?config.auth.method,
                    "User not authenticated"
                );
                return Err(GatewayError::Unauthorized);
            }
        }
    }
    Ok(())
}

#[inline(always)]
pub fn extract(
    config: &Config,
    headers: &hyper::HeaderMap<axum::http::HeaderValue>,
) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            let prefix = &config.auth.prefix;
            s.strip_prefix(prefix.as_str())?.strip_prefix(' ')
        })
        .map(str::to_owned)
}
