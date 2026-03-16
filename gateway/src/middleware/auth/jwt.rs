use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::error::GatewayError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // keeping it optional and passing null to header
    // might be best way incase use does not need fields
    pub user_id: Option<String>,
    pub role: Option<String>,
    pub exp: usize,
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
pub fn decode(
    token: &str,
    secret: &String,
    algorithm: &Algorithm,
) -> Result<Claims, GatewayError> {
    match jsonwebtoken::decode::<Claims>(
        token,
        //TODO: change to not create a decoding key each time
        //      keeping it in app_state would be better.
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(*algorithm),
    ) {
        Ok(decoded) => Ok(decoded.claims),
        Err(_) => Err(GatewayError::Unauthorized),
    }
}
