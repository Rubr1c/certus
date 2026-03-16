use crate::{
    config::types::{AuthType, Config},
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
#[inline]
pub fn run(
    headers: &mut hyper::HeaderMap<axum::http::HeaderValue>,
    token: Option<&str>,
    config: &Config,
    needs_auth: bool,
) -> Result<(), GatewayError> {
    if needs_auth {
        tracing::info!("Authenticating user");
        match token {
            Some(t) => {
                match &config.auth.method {
                    AuthType::JWT { secret, algorithm } => {
                        match jwt::decode(t, secret, &algorithm) {
                            Ok(claims) => {
                                //TODO: put claims in header
                                tracing::info!("User authenticated");

                                match claims.user_id {
                                    Some(id) => {
                                        headers.insert(
                                        "X-User-Id",
                                        axum::http::HeaderValue::from_str(
                                            id.as_str(),
                                        )
                                        .map_err(|_| GatewayError::InternalServerError)?
                                    );
                                    }
                                    _ => {}
                                }

                                match claims.role {
                                    Some(role) => {
                                        headers.insert(
                                        "X-User-Role",
                                        axum::http::HeaderValue::from_str(
                                            role.as_str(),
                                        )
                                        .map_err(|_| GatewayError::InternalServerError)?
                                    );
                                    }
                                    _ => {}
                                }

                                return Ok(());
                            }
                            Err(e) => {
                                tracing::info!("User not authenticated");
                                return Err(e);
                            }
                        }
                    }
                    AuthType::None => return Ok(()),
                }
            }
            _ => {
                tracing::info!("User not authenticated");
                return Err(GatewayError::Unauthorized);
            }
        }
    }
    return Ok(());
}
