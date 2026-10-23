use crate::network::error::{NetworkError, Result};
use quinn::{ServerConfig as QuinnServerConfig, ClientConfig as QuinnClientConfig};
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerifier, ServerCertVerified};
use rustls::{DigitallySignedStruct, SignatureScheme};
use std::sync::Arc;
use crate::network::NetworkConfig;

const CERT_DER: &[u8] = include_bytes!("../../../../main/rust/assets/cert.der");
const KEY_DER: &[u8] = include_bytes!("../../../../main/rust/assets/key.der");

pub struct CryptoManager {
    client_roots: Arc<rustls::RootCertStore>,
    cert_der: CertificateDer<'static>,
    key_der_bytes: Vec<u8>,
}

impl CryptoManager {
    pub fn new() -> Result<Self> {
        let cert_der = CertificateDer::from(CERT_DER.to_vec());
        let key_der_bytes = KEY_DER.to_vec();
        let client_roots = Self::build_root_store(&cert_der)?;

        Ok(Self {
            client_roots: Arc::new(client_roots),
            cert_der,
            key_der_bytes,
        })
    }

    pub fn server_config(&self, config: &NetworkConfig) -> Result<QuinnServerConfig> {
        let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key_der_bytes.clone()));
        Self::build_server_config(self.cert_der.clone(), key_der, config.idle_timeout_ms, config.keep_alive_interval_ms)
    }

    pub fn client_roots(&self) -> Arc<rustls::RootCertStore> {
        Arc::clone(&self.client_roots)
    }

    fn build_root_store(cert_der: &CertificateDer<'static>) -> Result<rustls::RootCertStore> {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(cert_der.clone())
            .map_err(|e| NetworkError::Certificate(format!("Root store add failed: {e}")))?;
        Ok(roots)
    }

    fn build_server_config(
            cert_der: CertificateDer<'static>,
            key_der: PrivateKeyDer<'static>,
            // Можно передавать duration аргументами или брать из конфига
            idle_timeout_ms: u64,
            keep_alive_ms: u64,
    ) -> Result<QuinnServerConfig> {
        let mut server_config = QuinnServerConfig::with_single_cert(vec![cert_der], key_der)
            .map_err(|e| NetworkError::Tls(format!("ServerConfig failed: {e}")))?;

        let mut transport_config = quinn::TransportConfig::default();

        let idle_timeout = std::time::Duration::from_millis(idle_timeout_ms)
            .try_into()
            .unwrap();
        transport_config.max_idle_timeout(Some(idle_timeout));

        transport_config.keep_alive_interval(Some(std::time::Duration::from_millis(keep_alive_ms)));

        server_config.transport_config(Arc::new(transport_config));

        Ok(server_config)
    }
}

#[derive(Debug)]
struct SkipHostnameVerifier {
    roots: Arc<rustls::RootCertStore>,
}

impl ServerCertVerifier for SkipHostnameVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ED25519,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}

pub fn make_client_config(roots: Arc<rustls::RootCertStore>) -> Result<QuinnClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipHostnameVerifier { roots }))
        .with_no_client_auth();

    let quic_client_config = QuicClientConfig::try_from(Arc::new(crypto))
        .map_err(|e| NetworkError::Tls(format!("QuicClientConfig failed: {e}")))?;
    let mut quinn_config = QuinnClientConfig::new(Arc::new(quic_client_config));
    quinn_config.transport_config(Arc::new({
        let mut config = quinn::TransportConfig::default();
        config.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into().unwrap()));
        config.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
        config
    }));
    
    Ok(quinn_config)
}