use crate::errors::*;

use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use crate::mqtt::MqttOptions;
use std::sync::Arc;
use url::Url;

pub fn mqtt_connect(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("mqtt_connect", hlua::function2(move |url: String, options: AnyLuaValue| -> Result<String> {
        let options = MqttOptions::try_from(options)
            .context("Invalid mqtt options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        let url = Url::parse(&url)
            .context("Failed to parse url")
            .map_err(|err| state.set_error(Error::from(err)))?;

        state.mqtt_connect(url, &options)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn mqtt_subscribe(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("mqtt_subscribe", hlua::function3(move |sock: String, topic: String, level: u8| -> Result<()> {
        let sock = state.get_mqtt(&sock);
        let mut sock = sock.lock().unwrap();

        sock.subscribe(&topic, level)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn mqtt_recv(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("mqtt_recv", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_mqtt(&sock);
        let mut sock = sock.lock().unwrap();

        let pkt = sock.recv_pkt()
            .map_err(|err| state.set_error(err))?;

        if let Some(pkt) = pkt {
            pkt.to_lua()
                .map_err(|err| state.set_error(err))
        } else {
            Ok(AnyLuaValue::LuaNil)
        }
    }))
}

pub fn mqtt_ping(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("mqtt_ping", hlua::function1(move |sock: String| -> Result<()> {
        let sock = state.get_mqtt(&sock);
        let mut sock = sock.lock().unwrap();

        sock.ping()
            .map_err(|err| state.set_error(err))
    }))
}
