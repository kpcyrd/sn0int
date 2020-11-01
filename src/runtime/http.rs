use crate::errors::*;

use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue, AnyHashableLuaValue};
use crate::json;
use sn0int_std::blobs::BlobState;
use sn0int_std::web::WebState;
use std::sync::Arc;
use std::collections::HashMap;
use crate::web::{RequestOptions, HttpRequest};


pub fn http_mksession(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("http_mksession", hlua::function0(move || -> String {
        state.http_mksession()
    }))
}

pub fn http_request(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("http_request", hlua::function4(move |session: String, method: String, url: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        RequestOptions::try_from(options)
            .context("invalid request options")
            .map_err(|err| state.set_error(Error::from(err)))
            .map(|options| {
                state.http_request(&session, method, url, options).into()
            })
    }))
}

pub fn http_send<S>(lua: &mut hlua::Lua, state: Arc<S>)
    where S: State + WebState + BlobState + 'static
{
    lua.set("http_send", hlua::function1(move |request: AnyLuaValue| -> Result<HashMap<AnyHashableLuaValue, AnyLuaValue>> {
        let mut req = HttpRequest::try_from(request)
            .context("invalid http request object")
            .map_err(|err| state.set_error(err.into()))?;

        let resp = req.send(state.as_ref())
            .map_err(|err| state.set_error(err))?;

        req.response_to_lua(state.as_ref(), resp)
            .map_err(|err| state.set_error(err))
            .map(|resp| resp.into())
    }))
}

pub fn http_fetch<S>(lua: &mut hlua::Lua, state: Arc<S>)
    where S: State + WebState + BlobState + 'static
{
    lua.set("http_fetch", hlua::function1(move |request: AnyLuaValue| -> Result<AnyLuaValue> {
        let mut req = HttpRequest::try_from(request)
            .context("invalid http request object")
            .map_err(|err| state.set_error(err.into()))?;

        let resp = req.send(state.as_ref())
            .map_err(|err| state.set_error(err))?;

        if resp.status < 200 || resp.status > 299 {
            return Err(state.set_error(format_err!("http status error: {}", resp.status)));
        }

        req.response_to_lua(state.as_ref(), resp)
            .map_err(|err| state.set_error(err))
            .map(|resp| resp.into())
    }))
}

pub fn http_fetch_json<S>(lua: &mut hlua::Lua, state: Arc<S>)
    where S: State + WebState + 'static
{
    lua.set("http_fetch_json", hlua::function1(move |request: AnyLuaValue| -> Result<AnyLuaValue> {
        let mut req = HttpRequest::try_from(request)
            .context("invalid http request object")
            .map_err(|err| state.set_error(err.into()))?;

        let resp = req.send(state.as_ref())
            .map_err(|err| state.set_error(err))?;

        if resp.status < 200 || resp.status > 299 {
            return Err(state.set_error(format_err!("http status error: {}", resp.status)));
        }

        json::decode(&resp.body)
            .map_err(|err| state.set_error(err))
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;
    use std::time::{Instant, Duration};

    #[test]
    #[ignore]
    fn verify_request() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/anything", {})
            x = http_send(req)
            if last_err() then return end
            print(x)

            if x['status'] ~= 200 then
                return 'wrong status code'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_timeout() {
        let start = Instant::now();
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "http://1.2.3.4", {
                timeout=250
            })
            x = http_send(req)
            if last_err() then return end
        end
        "#).expect("failed to load script");
        script.test().err().expect("Script should have failed");
        let end = Instant::now();

        assert!(end.duration_since(start) < Duration::from_secs(1));
    }

    #[test]
    #[ignore]
    fn verify_post() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()

            headers = {}
            headers['Content-Type'] = "application/json"
            req = http_request(session, "POST", "https://httpbin.org/anything", {
                headers=headers,
                query={
                    foo="bar"
                },
                json={
                    hello="world"
                }
            })
            x = http_send(req)
            if last_err() then return end
            print(x)

            o = json_decode(x['text'])
            if last_err() then return end

            if o['args']['foo'] ~= 'bar' or o['json']['hello'] ~= 'world' then
                return "reply didn't contain all params"
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_cookies() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()

            req = http_request(session, "GET", "https://httpbin.org/cookies/set", {
                query={
                    foo="bar",
                    fizz="buzz"
                }
            })
            x = http_send(req)

            req = http_request(session, "GET", "https://httpbin.org/cookies", {})
            x = http_send(req)
            if last_err() then return end
            print(x)

            o = json_decode(x['text'])
            if last_err() then return end

            if o['cookies']['fizz'] ~= 'buzz' or o['cookies']['foo'] ~= 'bar' then
                return "reply didn't contain all cookies"
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_ok() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/anything", {})
            x = http_fetch(req)
            if last_err() then return end

            o = json_decode(x['text'])
            if last_err() then return end

            if o['method'] ~= 'GET' then
                return 'unexpected response'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_json_ok() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/anything", {})
            x = http_fetch_json(req)
            if last_err() then return end
            print(x)
            if x['method'] ~= 'GET' then
                return 'unexpected response'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_404() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/status/404", {})
            x = http_fetch(req)
        end
        "#).expect("failed to load script");
        script.test().err().expect("Script should have failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_json_404() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/status/404", {})
            x = http_fetch_json(req)
        end
        "#).expect("failed to load script");
        script.test().err().expect("Script should have failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_json_invalid() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/html", {})
            x = http_fetch_json(req)
        end
        "#).expect("failed to load script");
        script.test().err().expect("Script should have failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_redirects() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "http://github.com", {
                follow_redirects=1,
            })
            x = http_send(req)
            if last_err() then return end

            if x['status'] ~= 200 then
                return 'redirect wasn\'t followed'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_fetch_skip_redirects() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, "GET", "http://github.com", {
                follow_redirects=0,
            })
            x = http_send(req)
            if last_err() then return end

            if x['status'] ~= 301 then
                return 'redirect was followed'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }
}
