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
            tracing::info!(address = %address, "Starting HTTPS listener");
            server::tls::serve(app, address, server::tls::load(conf).await)
                .await;
        }
        _ => {
            tracing::info!(address = %address, "Starting HTTP listener");
            let listener = match tokio::net::TcpListener::bind(address).await {
                Ok(listener) => listener,
                Err(err) => {
                    tracing::error!(
                        address = %address,
                        err = ?err,
                        "Failed to bind TCP listener"
                    );
                    panic!("failed to bind tcp listener");
                }
            };

            if let Err(err) = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown::signal())
            .await
            {
                tracing::error!(
                    address = %address,
                    err = ?err,
                    "HTTP server exited with error"
                );
                panic!("http server exited with error");
            }
        }
    }
}
