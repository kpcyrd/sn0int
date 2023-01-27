use crate::errors::*;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use rustls::{self, ClientConfig, Session, ClientSession};
use serde::Serialize;
use std::str;
use std::result;
use std::sync::Arc;
use std::net::TcpStream;
use super::{Stream, SocketOptions};

#[derive(Debug, Serialize)]
pub struct TlsData {
    cert: Option<String>,
    cert_chain: Vec<String>,
}

impl TlsData {
    pub fn to_lua(&self) -> Result<AnyLuaValue> {
        let v = serde_json::to_value(self)?;
        let v = LuaJsonValue::from(v).into();
        Ok(v)
    }
}

pub fn wrap_if_enabled(stream: TcpStream, host: &str, options: &SocketOptions) -> Result<Stream> {
    if !options.tls {
        let stream = Stream::Tcp(stream);
        Ok(stream)
    } else {
        let (socket, _) = wrap(stream, host, options)?;
        Ok(socket)
    }
}

pub fn wrap(stream: TcpStream, host: &str, options: &SocketOptions) -> Result<(Stream, TlsData)> {
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    config.ct_logs = Some(&ct_logs::LOGS);

    if options.disable_tls_verify {
        info!("tls verification has been disabled");
        config.dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
    }

    let dns_name = if let Some(v) = &options.sni_value {
        get_dns_name(&mut config, v)
    } else {
        get_dns_name(&mut config, host)
    };

    let config = Arc::new(config);
    let session = ClientSession::new(&config, dns_name.as_ref());
    setup(stream, session)
}

fn get_dns_name(config: &mut ClientConfig, host: &str) -> webpki::DNSName {
    if let Ok(name) = webpki::DNSNameRef::try_from_ascii_str(host) {
        debug!("setting sni value to: {:?}", host);
        name.to_owned()
    } else {
        debug!("sni extension has been disabled");
        config.enable_sni = false;
        webpki::DNSNameRef::try_from_ascii_str("invalid.com")
            .unwrap()
            .to_owned()
    }
}

fn setup(mut stream: TcpStream, mut session: ClientSession) -> Result<(Stream, TlsData)> {
    info!("starting tls handshake");
    if session.is_handshaking() {
        session.complete_io(&mut stream)
            .context("Failed to read reply to tls client hello")?;
    }

    if session.wants_write() {
        session.complete_io(&mut stream)
            .context("wants_write->complete_io failed")?;
    }

    let mut tls = TlsData {
        cert: None,
        cert_chain: Vec::new(),
    };

    if let Some(certs) = session.get_peer_certificates() {
        tls.cert_chain = certs.into_iter()
            .rev()
            .map(|c| {
                pem::encode(&pem::Pem {
                    tag: String::from("CERTIFICATE"),
                    contents: c.0,
                })
            })
            .collect();
    }

    tls.cert = tls.cert_chain.last()
        .map(|x| x.to_owned());

    info!("successfully established tls connection");
    let stream = rustls::StreamOwned::new(session, stream);
    let stream = Stream::Tls(Box::new(stream));
    Ok((stream, tls))
}

pub struct NoCertificateVerification {}

impl rustls::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(&self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef<'_>,
        _ocsp: &[u8]) -> result::Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}
