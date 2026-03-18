use std::sync::Arc;

use rustls::{
    ClientConfig, SignatureScheme,
    client::danger::{
        HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
    },
    crypto::CryptoProvider,
    pki_types,
};
use tokio_rustls::TlsConnector;

use crate::upstream::protocol;

// temporary

#[derive(Debug)]
struct NoVerifier(Arc<CryptoProvider>);

impl ServerCertVerifier for NoVerifier {
    #[inline(always)]
    fn verify_server_cert(
        &self,
        _end_entity: &pki_types::CertificateDer<'_>,
        _intermediates: &[pki_types::CertificateDer<'_>],
        _server_name: &pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: pki_types::UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    #[inline(always)]
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    #[inline(always)]
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    #[inline(always)]
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[inline(always)]
pub fn tls_connector(http_version: protocol::HttpVersion) -> TlsConnector {
    let provider =
        CryptoProvider::get_default().cloned().unwrap_or_else(|| {
            Arc::new(rustls::crypto::aws_lc_rs::default_provider())
        });

    let mut config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier(provider)))
        .with_no_client_auth();

    config.alpn_protocols = match http_version {
        protocol::HttpVersion::HTTP2 => vec![b"h2".to_vec()],
        protocol::HttpVersion::HTTP1 => vec![b"http/1.1".to_vec()],
    };

    TlsConnector::from(Arc::new(config))
}
