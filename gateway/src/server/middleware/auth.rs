use axum::http::HeaderValue;
use hyper::HeaderMap;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    config::{AuthType, Config},
    server::error::GatewayError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // keeping it optional and passing null to header
    // might be best way incase use does not need fields
    user_id: Option<String>,
    role: Option<String>,
    exp: usize,
}

/// Decodes a jwt token from a secret
///
/// # Arguments
///
/// * `token` - token to decode
/// * `secret` - secret to use for decoding
///
/// # Errors
///
/// Returns an error if:
/// * Failed to decode token
/// * Token is expired
#[inline]
pub fn decode(token: &str, secret: &String) -> Result<Claims, GatewayError> {
    match jsonwebtoken::decode::<Claims>(
        token,
        //TODO: change to not create a decoding key each time
        //      keeping it in app_state would be better.
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(decoded) => Ok(decoded.claims),
        Err(_) => Err(GatewayError::Unauthorized),
    }
}

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
    headers: &mut HeaderMap<HeaderValue>,
    token: Option<&str>,
    config: &Config,
    needs_auth: bool,
) -> Result<(), GatewayError> {
    //TODO: strip any prefix and define in config
    if needs_auth {
        tracing::info!("Authenticating user");
        match token {
            Some(t) => match &config.auth.method {
                AuthType::JWT { secret } => {
                    match decode(t, secret) {
                        Ok(claims) => {
                            //TODO: put claims in header
                            tracing::info!("User authenticated");

                            match claims.user_id {
                                Some(id) => {
                                    headers.insert(
                                        "X-User-Id",
                                        HeaderValue::from_str(id.as_str())
                                            .map_err(|_| GatewayError::InternalServerError)?
                                    );
                                }
                                _ => {}
                            }

                            match claims.role {
                                Some(role) => {
                                    headers.insert(
                                        "X-User-Role",
                                        HeaderValue::from_str(role.as_str())
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
            },
            _ => {
                tracing::info!("User not authenticated");
                return Err(GatewayError::Unauthorized);
            }
        }
    }
    return Ok(());
}
