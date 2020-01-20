use chrootable_https::DnsResolver;
use crate::errors::*;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use crate::sockets::{Stream, SocketOptions};
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use tungstenite::handshake::client::Request;
use tungstenite::protocol::{self, Message};
use url::Url;

#[derive(Debug, Default, Deserialize)]
pub struct WebSocketOptions {
    pub headers: Option<HashMap<String, String>>,
    pub proxy: Option<SocketAddr>,
}

impl WebSocketOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<WebSocketOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

pub struct WebSocket {
    sock: protocol::WebSocket<Stream>,
}

impl WebSocket {
    pub fn negotiate(stream: Stream, url: Url, headers: Option<&HashMap<String, String>>) -> Result<WebSocket> {
        let extra_headers = headers.map(|headers| {
            headers.iter()
                .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_str())))
                .collect()
        });

        let (sock, _resp) = tungstenite::client::client(
            Request {
                url,
                extra_headers,
            },
            stream,
        )?;
        Ok(WebSocket {
            sock,
        })
    }

    pub fn connect<R: DnsResolver>(resolver: &R, url: Url, options: &WebSocketOptions) -> Result<WebSocket> {
        let tls = match url.scheme() {
            "ws" => false,
            "wss" => true,
            _ => bail!("Invalid websocket protocol"),
        };

        let host = url.host_str()
            .ok_or_else(|| format_err!("Missing host in url"))?;

        let port = match (url.port(), tls) {
            (Some(port), _) => port,
            (None, true) => 443,
            (None, false) => 80,
        };

        let stream = Stream::connect_stream(resolver, host, port, &SocketOptions {
            tls,
            sni_value: None,
            disable_tls_verify: false,
            proxy: options.proxy,
        })?;
        Self::negotiate(stream, url, options.headers.as_ref())
    }

    fn read_msg(&mut self) -> Result<Option<Message>> {
        let msg = loop {
            let msg = match self.sock.read_message() {
                Ok(msg) => msg,
                Err(tungstenite::Error::ConnectionClosed) => break None,
                Err(tungstenite::Error::AlreadyClosed) => break None,
                Err(err) => return Err(err.into()),
            };
            match msg {
                Message::Text(_) | Message::Binary(_) => break Some(msg),
                Message::Ping(ping) => {
                    self.sock.write_message(Message::Pong(ping))?;
                },
                Message::Pong(_) => (), // this should never happen
                Message::Close(_) => break None,
            }
        };
        Ok(msg)
    }

    pub fn read_text(&mut self) -> Result<Option<String>> {
        match self.read_msg()? {
            Some(Message::Text(text)) => Ok(Some(text)),
            Some(_) => bail!("Unexpected message type: binary"),
            None => Ok(None),
        }
    }

    pub fn read_binary(&mut self) -> Result<Option<Vec<u8>>> {
        match self.read_msg()? {
            Some(Message::Binary(binary)) => Ok(Some(binary)),
            Some(_) => bail!("Unexpected message type: text"),
            None => Ok(None),
        }
    }

    fn write_msg(&mut self, msg: Message) -> Result<()> {
        self.sock.write_message(msg)?;
        self.sock.write_pending()?;
        Ok(())
    }

    pub fn write_text(&mut self, text: String) -> Result<()> {
        self.write_msg(Message::Text(text))
    }

    pub fn write_binary(&mut self, binary: Vec<u8>) -> Result<()> {
        self.write_msg(Message::Binary(binary))
    }
}
