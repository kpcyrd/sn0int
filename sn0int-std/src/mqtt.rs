use chrootable_https::DnsResolver;
use crate::errors::*;
use crate::hlua::AnyLuaValue;
use mqtt::packet::VariablePacketError;
use crate::json::LuaJsonValue;
use crate::sockets::{Stream, SocketOptions};
use mqtt::{TopicFilter, QualityOfService};
use mqtt::control::ConnectReturnCode;
use mqtt::control::fixed_header::FixedHeaderError;
use mqtt::encodable::{Encodable, Decodable};
use mqtt::packet::{VariablePacket, ConnectPacket, SubscribePacket, PingreqPacket};
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use std::io;
use std::net::SocketAddr;
use url::Url;

#[derive(Debug, Default, Deserialize)]
pub struct MqttOptions {
    pub username: Option<String>,
    pub password: Option<String>,

    pub proxy: Option<SocketAddr>,
    #[serde(default)]
    pub connect_timeout: u64,
    #[serde(default)]
    pub read_timeout: u64,
    #[serde(default)]
    pub write_timeout: u64,
}

impl MqttOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<MqttOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

pub struct MqttClient {
    stream: Stream,
}

impl MqttClient {
    pub fn negotiate(stream: Stream, options: &MqttOptions) -> Result<MqttClient> {
        let mut client = MqttClient {
            stream,
        };

        let mut pkt = ConnectPacket::new("sn0int");
        pkt.set_user_name(options.username.clone());
        pkt.set_password(options.password.clone());

        /*
        if let Some(keep_alive) = msg.keep_alive {
            packet.set_keep_alive(keep_alive);
        }
        */

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

    pub fn connect<R: DnsResolver>(resolver: &R, url: Url, options: &MqttOptions) -> Result<MqttClient> {
        let tls = match url.scheme() {
            "mqtt" => false,
            "mqtts" => true,
            _ => bail!("Invalid mqtt protocol"),
        };

        let host = url.host_str()
            .ok_or_else(|| format_err!("Missing host in url"))?;

        let port = match (url.port(), tls) {
            (Some(port), _) => port,
            (None, true) => 8883,
            (None, false) => 1883,
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

        Self::negotiate(stream, options)
    }

    fn send(&mut self, pkt: VariablePacket) -> Result<()> {
        debug!("Sending mqtt packet: {:?}", pkt);
        pkt.encode(&mut self.stream)?;
        Ok(())
    }

    fn recv(&mut self) -> std::result::Result<VariablePacket, VariablePacketError> {
        let pkt = VariablePacket::decode(&mut self.stream)?;
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
            Err(VariablePacketError::IoError(err)) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(VariablePacketError::FixedHeaderError(FixedHeaderError::IoError(err))) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(Error::from(err))
        }
    }

    pub fn ping(&mut self) -> Result<()> {
        let pkt = PingreqPacket::new();
        self.send(pkt.into())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Pkt {
    #[serde(rename="publish")]
    Publish(Publish),
    #[serde(rename="pong")]
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
