use rustls::client::danger;
use std::sync::Arc;

// pub fn client_config() -> rustls::ClientConfig {
//     use rustls::pki_types::{pem::PemObject, CertificateDer};
//     use std::sync::OnceLock;
//
//     static AWS_ROOT_BUNDLE: &[u8] = include_bytes!("../data/eu-north-1-bundle.pem");
//     static PROVIDER: OnceLock<Arc<rustls::crypto::CryptoProvider>> = OnceLock::new();
//     let provider = PROVIDER.get_or_init(|| Arc::new(rustls::crypto::ring::default_provider()));
//
//     let certs = CertificateDer::pem_slice_iter(AWS_ROOT_BUNDLE)
//         .collect::<std::result::Result<Vec<_>, _>>()
//         .expect("Loading embedded RootCerts");
//
//     let mut roots = rustls::RootCertStore::empty();
//     roots.add_parsable_certificates(certs);
//
//     rustls::ClientConfig::builder_with_provider(Arc::clone(provider))
//         .with_safe_default_protocol_versions()
//         .expect("versions")
//         .with_root_certificates(roots)
//         .with_no_client_auth()
// }

pub fn client_config() -> rustls::ClientConfig {
    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(CertVerifier))
        .with_no_client_auth()
}

#[derive(Debug)]
struct CertVerifier;

impl rustls::client::danger::ServerCertVerifier for CertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<danger::ServerCertVerified, rustls::Error> {
        Ok(danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
        Ok(danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
        Ok(danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
