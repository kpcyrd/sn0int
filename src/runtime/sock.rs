use crate::errors::*;

use crate::engine::ctx::State;
use crate::engine::structs::{byte_array, lua_bytes};
use crate::hlua::{self, AnyLuaValue};
use crate::sockets::SocketOptions;
use std::sync::Arc;


pub fn sock_connect(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_connect", hlua::function3(move |host: String, port: u16, options: AnyLuaValue| -> Result<String> {
        let options = SocketOptions::try_from(options)
            .context("Invalid socket options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        state.sock_connect(&host, port, &options)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sock_upgrade_tls(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_upgrade_tls", hlua::function2(move |sock: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        let options = SocketOptions::try_from(options)
            .context("Invalid socket options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        let tls = state.sock_upgrade_tls(&sock, &options)
            .map_err(|err| state.set_error(err))?;

        tls.to_lua()
    }))
}

pub fn sock_options(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_options", hlua::function2(move |sock: String, options: AnyLuaValue| -> Result<()> {
        let options = SocketOptions::try_from(options)
            .context("Invalid socket options")
            .map_err(|err| state.set_error(Error::from(err)))?;

        let sock = state.get_sock(&sock);
        let sock = sock.lock().unwrap();

        sock.options(&options)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_send(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_send", hlua::function2(move |sock: String, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.send(&bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recv(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recv", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recv()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendline(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_sendline", hlua::function2(move |sock: String, line: String| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.sendline(&line)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recvline(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvline", hlua::function1(move |sock: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline()
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvall(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvall", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvall()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvline_contains(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvline_contains", hlua::function2(move |sock: String, needle: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_contains(&needle)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvline_regex(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvline_regex", hlua::function2(move |sock: String, regex: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_regex(&regex)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvn(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvn", hlua::function2(move |sock: String, n: u32| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvn(n)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvuntil(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_recvuntil", hlua::function2(move |sock: String, delim: AnyLuaValue| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = sock.recvuntil(&delim)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendafter(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_sendafter", hlua::function3(move |sock: String, delim: AnyLuaValue, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.sendafter(&delim, &bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_newline(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("sock_newline", hlua::function2(move |sock: String, newline: String| {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.newline(newline);
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    #[ignore]
    fn verify_tcp_connect() {
        let script = Script::load_unchecked(r#"
        function run()
            sock = sock_connect('badssl.com', 443, {})
            if last_err() then return end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_tls_connect() {
        let script = Script::load_unchecked(r#"
        function run()
            sock = sock_connect('badssl.com', 443, {
                tls=true,
            })
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_tls_upgrade() {
        let script = Script::load_unchecked(r#"
        function run()
            sock = sock_connect('badssl.com', 443, {})
            if last_err() then return end
            sock_upgrade_tls(sock, {
                sni_value='badssl.com',
            })
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_tls_upgrade_expired() {
        let script = Script::load_unchecked(r#"
        function run()
            sock = sock_connect('expired.badssl.com', 443, {})
            if last_err() then return end
            sock_upgrade_tls(sock, {
                sni_value='expired.badssl.com',
            })
            if last_err() then
                -- this is supposed to fail
                clear_err()
            else
                return 'expired cert did not cause an error'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_tls_cert_chain_order() {
        let script = Script::load_unchecked(r#"
        function verify(crt)
            return crt['valid_names'][1] == 'badssl.com' or crt['valid_names'][2] == 'badssl.com'
        end

        function run()
            sock = sock_connect('badssl.com', 443, {})
            if last_err() then return end
            tls = sock_upgrade_tls(sock, {
                sni_value='badssl.com',
            })
            if last_err() then return end

            crt = x509_parse_pem(tls['cert'])
            if not verify(crt) then
                return 'unexpected certificate in tls[cert]'
            end

            crt = x509_parse_pem(tls['cert_chain'][#tls['cert_chain']])
            if not verify(crt) then
                return 'unexpected certificate in tls[cert_chain]'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_tls_connect_insecure() {
        let script = Script::load_unchecked(r#"
        function run()
            sock = sock_connect('expired.badssl.com', 443, {
                tls=true,
                disable_tls_verify=true,
            })
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }
}
