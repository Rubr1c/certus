use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;

use crate::{config::TLSConfig, server::shutdown};

#[inline(always)]
pub async fn load(config: TLSConfig) -> RustlsConfig {
    tracing::info!(
        cert_path = %config.cert_path,
        key_path = %config.key_path,
        "Loading TLS certificate files"
    );
    match RustlsConfig::from_pem_file(&config.cert_path, &config.key_path).await
    {
        Ok(tls_config) => tls_config,
        Err(err) => {
            tracing::error!(
                cert_path = %config.cert_path,
                key_path = %config.key_path,
                err = ?err,
                "Failed to load TLS certificate files"
            );
            panic!("invalid tls configuration");
        }
    }
}

#[inline(always)]
pub async fn serve(
    app: axum::Router,
    address: SocketAddr,
    tls_config: RustlsConfig,
) {
    tracing::info!(address = %address, "Serving HTTPS traffic");
    if let Err(err) = axum_server::bind_rustls(address, tls_config)
        .handle(shutdown::handle())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        tracing::error!(address = %address, err = ?err, "HTTPS server exited with error");
        panic!("https server exited with error");
    }
}
