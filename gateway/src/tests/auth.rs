use axum::body::Body;
use hyper::Request;

use crate::{
    config::{AuthConfig, AuthType, Config},
    server::middleware::auth,
};

fn dummy_req() -> Request<Body> {
    Request::builder().body(Body::empty()).unwrap()
}

#[test]
fn auth_not_enabled() {
    let config = Config::default();
    let mut req = dummy_req();

    let res = auth::run(req.headers_mut(), None, &config, false);
    assert!(res.is_ok());
}

#[test]
fn auth_enabled_no_token() {
    let mut config = Config::default();
    config.auth = AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
        prefix: "Bearer".to_string(),
    };
    let mut req = dummy_req();

    let res = auth::run(req.headers_mut(), None, &config, true);
    assert!(res.is_err());
}

#[test]
fn auth_enabled_invalid_token() {
    let mut config = Config::default();
    config.auth = AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
        prefix: "Bearer".to_string(),
    };
    let mut req = dummy_req();

    let res = auth::run(req.headers_mut(), Some("wdundw"), &config, true);
    assert!(res.is_err());
}

#[test]
fn auth_enabled_valid_token() {
    let mut config = Config::default();
    config.auth = AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
        prefix: "Bearer".to_string(),
    };

    let mut req = dummy_req();

    let res = auth::run(
        req.headers_mut(),
        Some(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNzcxNDE2NzEzLCJleHAiOjk5OTk5OTk5OTk5OTk5OX0.pc2WtJqpDMbWWvOrEOjPcQRkwJD2rxphmf-glLtyxqM",
        ),
        &config,
        true,
    );
    assert!(res.is_ok());
}
