use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    config::{AuthType, Config},
    server::{error::GatewayError, upstream::UpstreamServer},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // keeping it optional and passing null to header
    // might be best way incase use does not need fields
    user_id: Option<String>,
    role: Option<String>,
    exp: usize,
}

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

#[inline]
pub fn run(
    upstream: &UpstreamServer,
    config: &Config,
    token: Option<&str>,
) -> Result<(), GatewayError> {
    //TODO: strip any prefix and define in config
    if let Some(ref a) = config.auth {
        if upstream.req_auth {
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
