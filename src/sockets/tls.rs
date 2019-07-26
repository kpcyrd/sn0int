use crate::errors::*;

use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use rustls::{self, ClientConfig, Session, ClientSession, RootCertStore};

use std::str;
use std::sync::Arc;
use std::net::TcpStream;

use super::{Socket, Stream, SocketOptions};

#[derive(Debug, Serialize)]
pub struct TlsData {
// ClientSession::get_peer_certificates
}

impl TlsData {
    pub fn to_lua(&self) -> Result<AnyLuaValue> {
        let v = serde_json::to_value(&self)?;
        let v = LuaJsonValue::from(v).into();
        Ok(v)
    }
}

pub fn wrap_if_enabled(stream: TcpStream, host: &str, options: &SocketOptions) -> Result<Socket> {
    if !options.tls {
        let stream = Stream::Tcp(stream);
        return Ok(Socket::new(stream));
    } else {
        let (socket, _) = wrap(stream, host, options)?;
        Ok(socket)
    }
}

pub fn wrap(stream: TcpStream, host: &str, options: &SocketOptions) -> Result<(Socket, TlsData)> {
    let mut anchors = RootCertStore::empty();
    anchors.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let mut config = ClientConfig::new();
    config.root_store = anchors;

    let dns_name = if let Some(v) = &options.sni_value {
        get_dns_name(&mut config, &v)
    } else {
        get_dns_name(&mut config, &host)
    };

    let config = Arc::new(config);
    let session = ClientSession::new(&config, dns_name.as_ref());
    let stream = setup(stream, session)?;

    let tls = TlsData { }; // TODO

    Ok((Socket::new(stream), tls))
}

fn get_dns_name(config: &mut ClientConfig, host: &str) -> webpki::DNSName {
    if let Ok(name) = webpki::DNSNameRef::try_from_ascii_str(&host) {
        name.to_owned()
    } else {
        config.enable_sni = false;
        webpki::DNSNameRef::try_from_ascii_str("invalid.com")
            .unwrap()
            .to_owned()
    }
}

fn setup(mut stream: TcpStream, mut session: ClientSession) -> Result<Stream> {
    if session.is_handshaking() {
        session.complete_io(&mut stream)?;
    }

    if session.wants_write() {
        session.complete_io(&mut stream)?;
    }

    let stream = rustls::StreamOwned::new(session, stream);
    Ok(Stream::Tls(stream))
}
