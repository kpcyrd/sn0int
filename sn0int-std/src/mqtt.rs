use crate::errors::*;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use crate::sockets::{SocketOptions, Stream};
use chrootable_https::DnsResolver;
use mqtt::control::fixed_header::FixedHeaderError;
use mqtt::control::ConnectReturnCode;
use mqtt::encodable::{Decodable, Encodable};
use mqtt::packet::VariablePacketError;
use mqtt::packet::{ConnectPacket, PingreqPacket, SubscribePacket, VariablePacket};
use mqtt::{QualityOfService, TopicFilter};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use url::Url;

// a reasonable default for keep-alive
// some servers reject 0 as invalid with a very confusing error message
const DEFAULT_PING_INTERVAL: u64 = 90;
const DEFAULT_KEEP_ALIVE: u16 = 120;

#[derive(Debug, Default, Deserialize)]
pub struct MqttOptions {
    pub username: Option<String>,
    pub password: Option<String>,

    pub proxy: Option<SocketAddr>,
    #[serde(default)]
    pub connect_timeout: u64,
    pub read_timeout: Option<u64>,
    #[serde(default)]
    pub write_timeout: u64,

    pub ping_interval: Option<u64>,
    pub keep_alive: Option<u16>,
}

impl MqttOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<MqttOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MqttRecvError {
    #[error("Failed to read mqtt packet: {0:#}")]
    Recv(#[from] VariablePacketError),
    #[error("Failed to read mqtt packet: connection disconnected")]
    RecvDisconnect,
    #[error("Failed to interact with mqtt: {0:#}")]
    Error(Error),
}

impl From<Error> for MqttRecvError {
    fn from(err: Error) -> Self {
        MqttRecvError::Error(err)
    }
}

pub struct MqttClient {
    stream: Stream,
    last_ping: Instant,
    ping_interval: Option<u64>,
}

impl MqttClient {
    pub fn negotiate(stream: Stream, options: &MqttOptions) -> Result<MqttClient> {
        // default to DEFAULT_PING_INTERVAL, if an explicit value of 0 was set, disable auto-ping
        let ping_interval = Some(options.ping_interval.unwrap_or(DEFAULT_PING_INTERVAL));
        ping_interval.filter(|s| *s != 0);

        let mut client = MqttClient {
            stream,
            last_ping: Instant::now(),
            ping_interval,
        };

        let mut pkt = ConnectPacket::new("sn0int");
        pkt.set_user_name(options.username.clone());
        pkt.set_password(options.password.clone());
        pkt.set_keep_alive(options.keep_alive.unwrap_or(DEFAULT_KEEP_ALIVE));

        client.send(pkt.into())?;
        let pkt = client.recv()?;

        if let VariablePacket::ConnackPacket(pkt) = pkt {
            let code = pkt.connect_return_code();
            if code == ConnectReturnCode::ConnectionAccepted {
                Ok(client)
            } else {
                bail!("MQTT negotiation failed: {:?}", code);
            }
        } else {
            bail!("Expected ConnAck, received {:?}", pkt);
        }
    }

    pub fn connect<R: DnsResolver>(
        resolver: &R,
        url: Url,
        options: &MqttOptions,
    ) -> Result<MqttClient> {
        let tls = match url.scheme() {
            "mqtt" => false,
            "mqtts" => true,
            _ => bail!("Invalid mqtt protocol"),
        };

        let host = url
            .host_str()
            .ok_or_else(|| format_err!("Missing host in url"))?;

        let port = match (url.port(), tls) {
            (Some(port), _) => port,
            (None, true) => 8883,
            (None, false) => 1883,
        };

        // if no read timeout is configured then keep alive won't work
        let read_timeout = options.read_timeout.unwrap_or(DEFAULT_PING_INTERVAL);

        let stream = Stream::connect_stream(
            resolver,
            host,
            port,
            &SocketOptions {
                tls,
                sni_value: None,
                disable_tls_verify: false,
                proxy: options.proxy,

                connect_timeout: options.connect_timeout,
                read_timeout,
                write_timeout: options.write_timeout,
            },
        )?;

        Self::negotiate(stream, options)
    }

    fn maintain_ping(&mut self) -> Result<()> {
        if let Some(ping_interval) = self.ping_interval {
            if self.last_ping.elapsed() >= Duration::from_secs(ping_interval) {
                self.ping().context("Failed to ping")?;
                self.last_ping = Instant::now();
            }
        }
        Ok(())
    }

    fn send(&mut self, pkt: VariablePacket) -> Result<()> {
        self.maintain_ping()?;
        debug!("Sending mqtt packet: {:?}", pkt);
        pkt.encode(&mut self.stream)?;
        Ok(())
    }

    fn recv(&mut self) -> std::result::Result<VariablePacket, MqttRecvError> {
        self.maintain_ping()?;
        let pkt = VariablePacket::decode(&mut self.stream).map_err(|err| match err {
            // search for any io error and check if it's ErrorKind::UnexpectedEof
            VariablePacketError::IoError(err)
            | VariablePacketError::FixedHeaderError(FixedHeaderError::IoError(err))
                if err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                MqttRecvError::RecvDisconnect
            }
            _ => MqttRecvError::Recv(err),
        })?;
        debug!("Received mqtt packet: {:?}", pkt);
        Ok(pkt)
    }

    pub fn subscribe(&mut self, topic: &str, qos: u8) -> Result<()> {
        let filter = TopicFilter::new(topic)?;

        let qos = match qos {
            0 => QualityOfService::Level0,
            1 => QualityOfService::Level1,
            2 => QualityOfService::Level2,
            _ => bail!("Invalid QoS level: {}", qos),
        };

        let pkt = SubscribePacket::new(1, vec![(filter, qos)]);
        self.send(pkt.into())?;

        let pkt = self.recv()?;
        if let VariablePacket::SubackPacket(_pkt) = pkt {
            Ok(())
        } else {
            bail!("Expected SubAck, received {:?}", pkt);
        }
    }

    pub fn recv_pkt(&mut self) -> Result<Option<Pkt>> {
        match self.recv() {
            Ok(pkt) => Ok(Some(Pkt::try_from(pkt)?)),
            // search for any io error and check if it's ErrorKind::WouldBlock
            Err(MqttRecvError::Recv(
                VariablePacketError::IoError(err)
                | VariablePacketError::FixedHeaderError(FixedHeaderError::IoError(err)),
            )) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn ping(&mut self) -> Result<()> {
        let pkt = PingreqPacket::new();
        let pkt = VariablePacket::PingreqPacket(pkt);
        pkt.encode(&mut self.stream)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Pkt {
    #[serde(rename = "publish")]
    Publish(Publish),
    #[serde(rename = "pong")]
    Pong,
}

impl Pkt {
    pub fn to_lua(&self) -> Result<AnyLuaValue> {
        let v = serde_json::to_value(self)?;
        let v = LuaJsonValue::from(v).into();
        Ok(v)
    }
}

impl TryFrom<VariablePacket> for Pkt {
    type Error = Error;

    fn try_from(pkt: VariablePacket) -> Result<Pkt> {
        match pkt {
            VariablePacket::ConnectPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::ConnackPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PublishPacket(pkt) => Ok(Pkt::Publish(Publish {
                topic: pkt.topic_name().to_string(),
                body: pkt.payload().to_vec(),
            })),
            VariablePacket::PubackPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PubrecPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PubrelPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PubcompPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PingreqPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::PingrespPacket(_) => Ok(Pkt::Pong),
            VariablePacket::SubscribePacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::SubackPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::UnsubscribePacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::UnsubackPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
            VariablePacket::DisconnectPacket(_) => bail!("Unsupported pkt: {:?}", pkt),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Publish {
    pub topic: String,
    pub body: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrootable_https::dns::Resolver;

    fn connect() -> Result<MqttClient> {
        let resolver = Resolver::from_system_v4().unwrap();
        let url = "mqtt://mqtt.winkekatze24.de".parse()?;
        MqttClient::connect(&resolver, url, &MqttOptions::default())
    }

    #[test]
    #[ignore]
    fn test_connect() {
        connect().expect("Failed to setup connection");
    }

    // this test is too flaky
    /*
    #[test]
    #[ignore]
    fn test_subscribe() {
        let mut c = connect().unwrap();
        c.subscribe("#", 0).unwrap();
    }
    */
}
