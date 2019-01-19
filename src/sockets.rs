use crate::errors::*;

use bufstream::BufStream;
use chrootable_https::dns::{DnsResolver, RecordType};
use chrootable_https::socks5::{self, ProxyDest};
use regex::Regex;
use tokio::runtime::Runtime;

use std::str;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr};


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

#[derive(Debug)]
pub struct Socket {
    stream: BufStream<TcpStream>,
    newline: String,
}

impl Socket {
    pub fn new(stream: TcpStream) -> Socket {
        let stream = BufStream::new(stream);
        Socket {
            stream,
            newline: String::from("\n"),
        }
    }

    pub fn connect<R: DnsResolver>(resolver: &R, host: &str, port: u16) -> Result<Socket> {
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
                    return Ok(Socket::new(socket));
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

    pub fn connect_socks5(proxy: &SocketAddr, host: &str, port: u16) -> Result<Socket> {
        debug!("connecting to {:?}:{:?} with socks5 on {:?}", host, port, proxy);

        let host = match host.parse::<Ipv4Addr>() {
            Ok(ipaddr) => ProxyDest::Ipv4Addr(ipaddr),
            _ => ProxyDest::Domain(host.to_string()),
        };

        let fut = socks5::connect(proxy, host, port);

        let mut rt = Runtime::new()?;
        let socket = rt.block_on(fut)?;

        let socket = unwrap_socket(socket)?;

        return Ok(Socket::new(socket));
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
