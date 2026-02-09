use axum::body::Body;
use hyper::Request;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    config::models::{AuthType, Config},
    server::{error::GatewayError, upstream::models::UpstreamServer},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // keeping it optional and passing null to header might be best way incase use does not need fields
    user_id: Option<String>,
    role: Option<String>,
    exp: usize,
}

#[inline]
pub fn decode(token: &str, secret: &String) -> Result<Claims, GatewayError> {
    match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(decoded) => Ok(decoded.claims),
        Err(_) => Err(GatewayError::Unauthorized),
    }
}

#[inline]
pub fn run(
    upstream: &UpstreamServer,
    req: &Request<Body>,
    config: &Config,
) -> Result<(), GatewayError> {
    //TODO: strip any prefix and define in config
    if let Some(ref a) = config.auth {
        if upstream.req_auth {
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "));

            match token {
                Some(t) => match &a.method {
                    AuthType::JWT { secret } => {
                        match decode(t, secret) {
                            Ok(_) => {
                                //TODO: put claims in header
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
    Ok(())
}
