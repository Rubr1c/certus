use crate::{
    config::{AuthConfig, AuthType, Config},
    server::middleware::auth,
};

#[test]
fn auth_not_enabled() {
    let config = Config::default();

    let res = auth::run(false, &config, None);
    assert!(res.is_ok());
}

#[test]
fn auth_enabled_no_token() {
    let mut config = Config::default();
    config.auth =
        AuthConfig { method: AuthType::JWT { secret: "secret".to_string() } };

    let res = auth::run(true, &config, None);
    assert!(res.is_err());
}

#[test]
fn auth_enabled_invalid_token() {
    let mut config = Config::default();
    config.auth =
        AuthConfig { method: AuthType::JWT { secret: "secret".to_string() } };

    let res = auth::run(true, &config, Some("wdundw"));
    assert!(res.is_err());
}

#[test]
fn auth_enabled_valid_token() {
    let mut config = Config::default();
    config.auth =
        AuthConfig { method: AuthType::JWT { secret: "secret".to_string() } };

    let res = auth::run(
        true,
        &config,
        Some(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNzcxNDE2NzEzLCJleHAiOjk5OTk5OTk5OTk5OTk5OX0.pc2WtJqpDMbWWvOrEOjPcQRkwJD2rxphmf-glLtyxqM",
        ),
    );
    assert!(res.is_ok());
}
