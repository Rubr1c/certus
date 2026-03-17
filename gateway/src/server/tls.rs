use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;

use crate::{config::TLSConfig, server::shutdown};

#[inline]
pub async fn load(config: TLSConfig) -> RustlsConfig {
    RustlsConfig::from_pem_file(&config.cert_path, &config.key_path)
        .await
        .expect("Invalid TLS")
}

#[inline]
pub async fn serve(
    app: axum::Router,
    address: SocketAddr,
    tls_config: RustlsConfig,
) {
    axum_server::bind_rustls(address, tls_config)
        .handle(shutdown::create_singal_handle())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}
