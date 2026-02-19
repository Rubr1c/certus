use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    config::AuthType,
    server::{
        app_state::AppState, error::GatewayError, upstream::UpstreamServer,
    },
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
pub fn decode(token: &str, key: &DecodingKey) -> Result<Claims, GatewayError> {
    match jsonwebtoken::decode::<Claims>(token, key, &Validation::default()) {
        Ok(decoded) => Ok(decoded.claims),
        Err(_) => Err(GatewayError::Unauthorized),
    }
}

/// Tries to authenticate the user if route requires auth
///
/// # Arguments
///
/// * `upstream` - server trying to connect to
/// * `config` - gateway config
/// * `token` - optional token if exists
///
/// # Errors
///
/// Returns an error if:
/// * Authentication is enabled and no token was provided
/// * Token failed to be decoded [`decode`]
#[inline]
pub fn run(
    upstream: &UpstreamServer,
    state: &AppState,
    token: Option<&str>,
) -> Result<(), GatewayError> {
    let config = state.config.load();

    //TODO: strip any prefix and define in config
    if upstream.req_auth {
        tracing::info!(?token, "Authenticating user");
        match token {
            Some(t) => match &config.auth.method {
                AuthType::JWT { secret: _ } => {
                    let key_guard = state.decoding_key.load();

                    let key_ref = key_guard.as_ref().as_ref();

                    match key_ref {
                        Some(key) => match decode(t, key) {
                            Ok(_) => {
                                tracing::info!("User authenticated");
                                return Ok(());
                            }
                            Err(e) => {
                                tracing::info!("User not authenticated");
                                return Err(e);
                            }
                        },
                        None => {
                            tracing::error!(
                                "JWT auth required but no decoding key is loaded"
                            );
                            return Err(GatewayError::Unauthorized);
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
