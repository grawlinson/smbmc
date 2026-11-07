//! TLS configuration: rustls with the `ring` provider, configured per client
//! so no process-wide default provider is installed.

use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, ring};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};

use crate::error::{Error, Result};

/// Build the rustls configuration handed to reqwest.
///
/// Certificates are verified with the platform verifier (the OS trust store),
/// unless `accept_invalid_certs` is set.
pub(crate) fn config(accept_invalid_certs: bool) -> Result<ClientConfig> {
    let provider = Arc::new(ring::default_provider());
    let builder = ClientConfig::builder_with_provider(Arc::clone(&provider))
        .with_safe_default_protocol_versions()
        .map_err(tls_error)?
        .dangerous();
    let builder = if accept_invalid_certs {
        builder.with_custom_certificate_verifier(Arc::new(NoVerifier(provider)))
    } else {
        let verifier = rustls_platform_verifier::Verifier::new(provider).map_err(tls_error)?;
        builder.with_custom_certificate_verifier(Arc::new(verifier))
    };
    Ok(builder.with_no_client_auth())
}

fn tls_error(e: rustls::Error) -> Error {
    Error::Tls(e.to_string())
}

/// Accepts any server certificate, but still checks handshake signatures so
/// the connection is at least consistent with the presented certificate.
#[derive(Debug)]
struct NoVerifier(Arc<CryptoProvider>);

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
