use std::net::SocketAddr;

#[inline(always)]
pub fn signal() -> impl Future<Output = ()> {
    async {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

        tracing::info!("\nShutting down...");
    }
}

#[inline(always)]
pub fn handle() -> axum_server::Handle<SocketAddr> {
    let handle = axum_server::Handle::new();
    let shutdown_handle = handle.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

        tracing::info!("\nShutting down...");
        shutdown_handle.graceful_shutdown(None);
    });

    handle
}
