use std::{sync::Arc, sync::atomic::Ordering, time::Duration};

use axum::body::Body;
use hyper::{Method, Request, client::conn, header};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::{
    ClientConfig, SignatureScheme,
    client::danger::{
        HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
    },
    crypto::CryptoProvider,
};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::instrument;

use crate::server::{
    error::GatewayError,
    middleware::handler,
    upstream::{HttpVersion, PooledConnection, Protocol, UpstreamServer},
};

#[derive(Debug)]
struct NoVerifier(Arc<CryptoProvider>);

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn tls_connector(http_version: HttpVersion) -> TlsConnector {
    let provider =
        CryptoProvider::get_default().cloned().unwrap_or_else(|| {
            Arc::new(rustls::crypto::aws_lc_rs::default_provider())
        });

    let mut config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier(provider)))
        .with_no_client_auth();

    config.alpn_protocols = match http_version {
        HttpVersion::HTTP2 => vec![b"h2".to_vec()],
        HttpVersion::HTTP1 => vec![b"http/1.1".to_vec()],
    };

    TlsConnector::from(Arc::new(config))
}

/// Opens a connection to a tcp stream
///
/// # Arguments
///
/// * `upstream` - target server to conenct to
/// * `timeout` - when to timeout trying to connect
///
/// # Errors
///
/// Returns an error if:
/// * Failed to connect to server
/// * Timed out while trying to connect
/// * Failed to perform handshake with server
#[instrument(skip_all, fields(http_version = ?upstream.pool.http_version))]
pub async fn open_connection(
    upstream: &UpstreamServer,
    timeout: u64,
) -> Result<PooledConnection, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Connecting to new upstream");
    let connect_future = TcpStream::connect(upstream.pool.server_addr.as_str());
    let stream =
        tokio::time::timeout(Duration::from_secs(timeout), connect_future)
            .await??;

    let sender = match upstream.pool.http_version {
        HttpVersion::HTTP1 => match upstream.pool.protocol {
            Protocol::HTTPS => {
                let connector = tls_connector(HttpVersion::HTTP1);
                let domain = rustls::pki_types::ServerName::try_from(
                    upstream.pool.hostname.as_str(),
                )?
                .to_owned();
                let tls_stream = connector.connect(domain, stream).await?;
                let io = TokioIo::new(tls_stream);
                let (sender, conn) =
                    conn::http1::handshake::<_, Body>(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::error!(?err, "Connection failed");
                    }
                });
                PooledConnection::Http1(sender)
            }
            Protocol::HTTP => {
                let io = TokioIo::new(stream);
                let (sender, conn) =
                    conn::http1::handshake::<_, Body>(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::error!(?err, "Connection failed");
                    }
                });
                PooledConnection::Http1(sender)
            }
        },
        HttpVersion::HTTP2 => {
            let exec = TokioExecutor::new();
            match upstream.pool.protocol {
                Protocol::HTTPS => {
                    let connector = tls_connector(HttpVersion::HTTP2);
                    let domain = rustls::pki_types::ServerName::try_from(
                        upstream.pool.hostname.as_str(),
                    )?
                    .to_owned();
                    let tls_stream = connector.connect(domain, stream).await?;
                    let io = TokioIo::new(tls_stream);
                    let (sender, conn) =
                        conn::http2::handshake(exec, io).await?;
                    tokio::task::spawn(async move {
                        if let Err(err) = conn.await {
                            tracing::error!(?err, "Connection failed");
                        }
                    });
                    PooledConnection::Http2(sender)
                }
                Protocol::HTTP => {
                    let io = TokioIo::new(stream);
                    let (sender, conn) =
                        conn::http2::handshake(exec, io).await?;
                    tokio::task::spawn(async move {
                        if let Err(err) = conn.await {
                            tracing::error!(?err, "Connection failed");
                        }
                    });
                    PooledConnection::Http2(sender)
                }
            }
        }
    };

    tracing::info!("Connetion made");

    Ok(sender)
}

/// Tries to borrow an idle connection if not able it
/// opens a new one
///
/// # Arguments
///
/// * `upstream` - target server to conenct to
/// * `timeout` - when to timeout trying to make a new connection
///
/// # Errors
///
/// Returns an error if:
/// * Max connections to server reached
/// * Failed to open new connection
pub async fn borrow_connection(
    upstream: &UpstreamServer,
    timeout: u64,
) -> Result<PooledConnection, GatewayError> {
    tracing::info!("Checking for idle connetions");
    while let Some(sender) = upstream.pool.idle_connections.pop() {
        let alive = match &sender {
            PooledConnection::Http1(s) => s.is_ready(),
            PooledConnection::Http2(s) => s.is_ready(),
        };

        if alive {
            tracing::info!("Found idle connetion");
            upstream.active_connctions.fetch_add(1, Ordering::AcqRel);
            return Ok(sender);
        }

        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
        tracing::warn!("Discarded dead idle connection");
    }

    tracing::info!("No idle connetions found");

    let total = upstream.pool.total_connections.load(Ordering::Acquire);
    if total >= upstream.pool.max_connections {
        tracing::info!(total_cons = total, "Max connetions reached");
        return Err(GatewayError::Overloaded);
    }

    let sender = open_connection(upstream, timeout)
        .await
        .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;

    upstream.pool.total_connections.fetch_add(1, Ordering::AcqRel);
    upstream.active_connctions.fetch_add(1, Ordering::AcqRel);

    Ok(sender)
}

/// Removes connection to server and keeps it idle if possible
///
/// # Arguments
///
/// * `upstream` - target server to release
/// * `sender` - connection to the server
/// * `reusable` - if the connection can be reused and put in idle
pub async fn release_connection(
    upstream: &UpstreamServer,
    sender: PooledConnection,
    reusable: bool,
) {
    tracing::info!("Releasing connetion");
    upstream.active_connctions.fetch_sub(1, Ordering::AcqRel);

    if reusable {
        upstream.pool.idle_connections.push(sender);
    } else {
        tracing::info!("Connetion not reusable");
        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Checks if the health of the server is ok
///
/// # Arguments
///
/// * `upstream` - target server trying to check
pub async fn health_ok(upstream: &UpstreamServer) -> bool {
    let req = match Request::builder()
        .method(Method::GET)
        .uri("/")
        .header(header::HOST, upstream.pool.hostname.as_str())
        .body(Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(err = ?e, "error building health req");
            return false;
        }
    };

    match handler::handle_request(&upstream, req, 2000).await {
        Ok(res) => {
            let status = res.status();
            status.is_success() || status.is_redirection()
        }
        Err(_) => false,
    }
}
