use crate::errors::*;

use bufstream::BufStream;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use chrootable_https::dns::{DnsResolver, RecordType};
use chrootable_https::socks5::{self, ProxyDest};
use regex::Regex;
use tokio::runtime::Runtime;

use std::fmt;
use std::str;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr};

mod tls;
pub use self::tls::TlsData;


#[cfg(unix)]
fn unwrap_socket(socket: tokio::net::TcpStream) -> Result<TcpStream> {
    use std::os::unix::io::AsRawFd;
    use std::os::unix::io::FromRawFd;
    let socket2 = unsafe { TcpStream::from_raw_fd(socket.as_raw_fd()) };
    let socket = socket2.try_clone()?;
    Ok(socket)
}

#[cfg(windows)]
fn unwrap_socket(socket: tokio::net::TcpStream) -> Result<TcpStream> {
    bail!("Unwrapping tokio sockets into std sockets isn't supported on windows")
}

#[derive(Debug, Default, Deserialize)]
pub struct SocketOptions {
    #[serde(default)]
    pub tls: bool,
    sni_value: Option<String>,
    #[serde(default)]
    disable_tls_verify: bool,

    // TODO: enable_sni (default to true)
    // TODO: cacert
    // TODO: timeout
}

impl SocketOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<SocketOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

#[derive(Debug)]
pub struct Socket {
    stream: BufStream<Stream>,
    newline: String,
}

enum Stream {
    Tcp(TcpStream),
    Tls(rustls::StreamOwned<rustls::ClientSession, TcpStream>),
}

impl fmt::Debug for Stream {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stream::Tcp(s) => write!(w, "Stream::Tcp {{ {:?} }}", s),
            Stream::Tls(_) => write!(w, "Stream::Tls {{ ... }}"),
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Stream::Tcp(s) => s.read(buf),
            Stream::Tls(s) => s.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Tcp(s) => s.write(buf),
            Stream::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Tcp(s) => s.flush(),
            Stream::Tls(s) => s.flush(),
        }
    }
}

impl Socket {
    fn new(stream: Stream) -> Socket {
        let stream = BufStream::new(stream);
        Socket {
            stream,
            newline: String::from("\n"),
        }
    }

    pub fn connect<R: DnsResolver>(resolver: &R, host: &str, port: u16, options: &SocketOptions) -> Result<Socket> {
        let addrs = match host.parse::<IpAddr>() {
            Ok(addr) => vec![addr],
            Err(_) => resolver.resolve(host, RecordType::A)
                .wait_for_response()?
                .success()?,
        };

        let mut errors = Vec::new();

        for addr in addrs {
            debug!("connecting to {}:{}", addr, port);
            match TcpStream::connect((addr, port)) {
                Ok(socket) => {
                    debug!("successfully connected to {:?}", addr);
                    return tls::wrap_if_enabled(socket, host, options);
                },
                Err(err) => errors.push((addr, err)),
            }
        }

        if errors.is_empty() {
            bail!("no dns records found");
        } else {
            bail!("couldn't connect: {:?}", errors);
        }
    }

    pub fn connect_socks5(proxy: &SocketAddr, host: &str, port: u16, options: &SocketOptions) -> Result<Socket> {
        debug!("connecting to {:?}:{:?} with socks5 on {:?}", host, port, proxy);

        let addr = match host.parse::<Ipv4Addr>() {
            Ok(ipaddr) => ProxyDest::Ipv4Addr(ipaddr),
            _ => ProxyDest::Domain(host.to_string()),
        };

        let fut = socks5::connect(proxy, addr, port);

        let mut rt = Runtime::new()?;
        let socket = rt.block_on(fut)?;

        let socket = unwrap_socket(socket)?;

        tls::wrap_if_enabled(socket, host, options)
    }

    pub fn upgrade_to_tls(self, options: &SocketOptions) -> Result<(Socket, TlsData)> {
        let stream = self.stream.into_inner()?;

        match stream {
            Stream::Tcp(stream) => tls::wrap(stream, "", options),
            _ => bail!("Only tcp streams can be upgraded"),
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        match str::from_utf8(&data) {
            Ok(data) => debug!("send: {:?}", data),
            Err(_) => debug!("send: {:?}", data),
        };
        self.stream.write_all(data)?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0; 4096];
        let n = self.stream.read(&mut buf)?;
        let data = buf[..n].to_vec();
        match str::from_utf8(&data) {
            Ok(data) => debug!("recv: {:?}", data),
            Err(_) => debug!("recv: {:?}", data),
        };
        Ok(data)
    }

    pub fn sendline(&mut self, line: &str) -> Result<()> {
        let line = format!("{}{}", line, self.newline);
        self.send(line.as_bytes())
    }

    pub fn recvline(&mut self) -> Result<String> {
        let needle = self.newline.clone();
        let buf = self.recvuntil(needle.as_bytes())?;
        let line = String::from_utf8(buf)
            .context("Failed to decode utf8")?;
        Ok(line)
    }

    pub fn recvall(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.stream.read_to_end(&mut buf)?;
        match str::from_utf8(&buf) {
            Ok(buf) => debug!("recvall: {:?}", buf),
            Err(_) => debug!("recvall: {:?}", buf),
        };
        Ok(buf)
    }

    pub fn recvline_contains(&mut self, needle: &str) -> Result<String> {
        loop {
            let line = self.recvline()?;
            if line.contains(needle) {
                return Ok(line);
            }
        }
    }

    pub fn recvline_regex(&mut self, regex: &str) -> Result<String> {
        let regex = Regex::new(regex)?;
        loop {
            let line = self.recvline()?;
            if regex.is_match(&line) {
                return Ok(line);
            }
        }
    }

    pub fn recvn(&mut self, n: u32) -> Result<Vec<u8>> {
        let mut buf = vec![0; n as usize];
        self.stream.read_exact(buf.as_mut_slice())?;
        match str::from_utf8(&buf) {
            Ok(buf) => debug!("recvn: {:?}", buf),
            Err(_) => debug!("recvn: {:?}", buf),
        };
        Ok(buf.to_vec())
    }

    pub fn recvuntil(&mut self, delim: &[u8]) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        let delim_len = delim.len();

        loop {
            let (done, used) = {
                let available = match self.stream.fill_buf() {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e.into())
                };

                match available.windows(delim_len).position(|window| window == delim) {
                    Some(i) => {
                        buf.extend_from_slice(&available[..i + delim_len]);
                        (true, i + delim_len)
                    }
                    None => {
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                }
            };
            self.stream.consume(used);

            if done || used == 0 {
                match str::from_utf8(&buf) {
                    Ok(buf) => debug!("recvuntil: {:?}", buf),
                    Err(_) => debug!("recvuntil: {:?}", buf),
                };
                return Ok(buf);
            }
        }
    }

    pub fn sendafter(&mut self, delim: &[u8], data: &[u8]) -> Result<()> {
        self.recvuntil(delim)?;
        self.send(data)
    }

    pub fn newline<I: Into<String>>(&mut self, delim: I) {
        self.newline = delim.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrootable_https::dns::Resolver;

    #[ignore]
    #[test]
    fn verify_tls_good() {
        let resolver = Resolver::from_system().unwrap();
        let _sock = Socket::connect(&resolver, "badssl.com", 443, &SocketOptions{
            tls: true,
            ..Default::default()
        }).unwrap();
    }

    #[test]
    #[ignore]
    fn verify_tls_good_request() {
        let resolver = Resolver::from_system().unwrap();
        let mut sock = Socket::connect(&resolver, "badssl.com", 443, &SocketOptions{
            tls: true,
            ..Default::default()
        }).unwrap();
        sock.send(b"GET / HTTP/1.1\r\nHost: badssl.com\r\nConnection: close\r\n\r\n").unwrap();
        let status = sock.recvline().unwrap();
        assert_eq!(status, "HTTP/1.1 200 OK\r\n");
    }

    #[test]
    #[ignore]
    fn verify_tls_expired() {
        let resolver = Resolver::from_system().unwrap();
        let sock = Socket::connect(&resolver, "expired.badssl.com", 443, &SocketOptions{
            tls: true,
            ..Default::default()
        });
        assert!(sock.is_err());
    }

    /*
    // https://github.com/ctz/rustls/issues/281

    #[test]
    #[ignore]
    fn verify_tls_1_1_1_1() {
        let resolver = Resolver::from_system().unwrap();
        let _sock = Socket::connect(&resolver, "1.1.1.1", 443, &SocketOptions{
            tls: true,
            ..Default::default()
        }).unwrap();
    }
    */
}
