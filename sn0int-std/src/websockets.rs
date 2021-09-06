use chrootable_https::DnsResolver;
use crate::errors::*;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use crate::sockets::{Stream, SocketOptions};
use http::Request;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::io;
use tungstenite::protocol::{self, Message};
use url::Url;

#[derive(Debug, Default, Deserialize)]
pub struct WebSocketOptions {
    pub headers: Option<HashMap<String, String>>,
    pub proxy: Option<SocketAddr>,

    #[serde(default)]
    pub connect_timeout: u64,
    #[serde(default)]
    pub read_timeout: u64,
    #[serde(default)]
    pub write_timeout: u64,
}

impl WebSocketOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<WebSocketOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

pub enum Event {
    Text(String),
    Binary(Vec<u8>),
    Close,
    Timeout,
}

pub struct WebSocket {
    sock: protocol::WebSocket<Stream>,
}

impl WebSocket {
    pub fn negotiate(stream: Stream, url: Url, headers: Option<&HashMap<String, String>>) -> Result<WebSocket> {
        let mut req = Request::get(url.to_string()); // TODO: don't re-parse here

        if let Some(headers) = headers {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        let req = req.body(()).unwrap();

        let (sock, _resp) = tungstenite::client::client(req, stream)?;
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

            connect_timeout: options.connect_timeout,
            read_timeout: options.read_timeout,
            write_timeout: options.write_timeout,
        })?;
        Self::negotiate(stream, url, options.headers.as_ref())
    }

    pub fn options(&self, options: &WebSocketOptions) -> Result<()> {
        let o = SocketOptions {
            read_timeout: options.read_timeout,
            write_timeout: options.write_timeout,
            ..Default::default()
        };
        o.apply(self.sock.get_ref())
    }

    fn read_msg(&mut self) -> Result<Event> {
        loop {
            let msg = match self.sock.read_message() {
                Ok(Message::Text(body)) => Event::Text(body),
                Ok(Message::Binary(body)) => Event::Binary(body),
                Ok(Message::Ping(ping)) => {
                    self.sock.write_message(Message::Pong(ping))?;
                    continue;
                },
                Ok(Message::Pong(_)) => continue, // this should never happen
                Ok(Message::Close(_)) => Event::Close,
                Err(tungstenite::Error::ConnectionClosed) => Event::Close,
                Err(tungstenite::Error::AlreadyClosed) => Event::Close,
                Err(tungstenite::Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => Event::Timeout,
                Err(err) => return Err(err.into()),
            };
            return Ok(msg);
        }
    }

    pub fn read_text(&mut self) -> Result<Option<String>> {
        match self.read_msg()? {
            Event::Text(text) => Ok(Some(text)),
            Event::Binary(_) => bail!("Unexpected message type: binary"),
            Event::Close => bail!("Connection closed"),
            Event::Timeout => Ok(None),
        }
    }

    pub fn read_binary(&mut self) -> Result<Option<Vec<u8>>> {
        match self.read_msg()? {
            Event::Text(_) => bail!("Unexpected message type: text"),
            Event::Binary(body) => Ok(Some(body)),
            Event::Close => bail!("Connection closed"),
            Event::Timeout => Ok(None),
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
