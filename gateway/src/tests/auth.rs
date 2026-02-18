use crate::{
    config::{AuthConfig, AuthType, Config},
    server::{
        app_state::AppState,
        middleware::auth,
        upstream::{Protocol, UpstreamServer},
    },
};

use super::create_socket_addr;

#[test]
fn auth_not_enabled() {
    let upstream = UpstreamServer::new(
        create_socket_addr(1)[0],
        100,
        Protocol::HTTP1,
        false,
    );

    let config = Config::default();

    let state = AppState::new(config);

    let res = auth::run(&upstream, &state, None);
    assert!(res.is_ok());
}

#[test]
fn auth_enabled_no_token() {
    let upstream = UpstreamServer::new(
        create_socket_addr(1)[0],
        100,
        Protocol::HTTP1,
        true,
    );

    let mut config = Config::default();
    config.auth = Some(AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
    });

    let state = AppState::new(config);

    let res = auth::run(&upstream, &state, None);
    assert!(res.is_err());
}

#[test]
fn auth_enabled_invalid_token() {
    let upstream = UpstreamServer::new(
        create_socket_addr(1)[0],
        100,
        Protocol::HTTP1,
        true,
    );

    let mut config = Config::default();
    config.auth = Some(AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
    });

    let state = AppState::new(config);

    let res = auth::run(&upstream, &state, Some("wdundw"));
    assert!(res.is_err());
}

#[test]
fn auth_enabled_valid_token() {
    let upstream = UpstreamServer::new(
        create_socket_addr(1)[0],
        100,
        Protocol::HTTP1,
        true,
    );

    let mut config = Config::default();
    config.auth = Some(AuthConfig {
        method: AuthType::JWT { secret: "secret".to_string() },
    });

    let state = AppState::new(config);

    let res = auth::run(
        &upstream,
        &state,
        Some(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNzcxNDE2NzEzLCJleHAiOjk5OTk5OTk5OTk5OTk5OX0.pc2WtJqpDMbWWvOrEOjPcQRkwJD2rxphmf-glLtyxqM",
        ),
    );
    assert!(res.is_ok());
}
