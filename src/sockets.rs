use crate::errors::*;

use bufstream::BufStream;
use regex::Regex;

use std::str;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::net::TcpStream;
use std::net::ToSocketAddrs;


#[derive(Debug)]
pub struct Socket {
    stream: BufStream<TcpStream>,
    newline: String,
}

impl Socket {
    pub fn connect(host: &str, port: u16) -> Result<Socket> {
        let addrs = (host, port).to_socket_addrs()?;

        let mut errors = Vec::new();

        for addr in addrs {
            debug!("connecting to {:?}", addr);
            match TcpStream::connect(&addr) {
                Ok(socket) => {
                    debug!("successfully connected to {:?}", addr);
                    let stream = BufStream::new(socket);

                    return Ok(Socket {
                        stream,
                        newline: String::from("\n"),
                    });
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
