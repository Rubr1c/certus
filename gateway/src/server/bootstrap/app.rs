use std::net::SocketAddr;

use crate::{
    config::TLSConfig,
    server::{self, shutdown},
};

#[inline]
pub async fn run(
    app: axum::Router,
    address: SocketAddr,
    tls_config: Option<TLSConfig>,
) {
    match tls_config {
        Some(conf) => {
            server::tls::serve(app, address, server::tls::load(conf).await)
                .await;
        }
        _ => {
            let listener = tokio::net::TcpListener::bind(address)
                .await
                .expect("Failed to bind TCP listener");

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown::create_signal())
            .await
            .unwrap();
        }
    }
}
