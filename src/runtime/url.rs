use crate::errors::*;
use crate::engine::ctx::State;
use crate::engine::structs::LuaMap;
use crate::hlua::{self, AnyLuaValue};
use crate::json::LuaJsonValue;
use url::Url;
use percent_encoding::{percent_encode, percent_decode, NON_ALPHANUMERIC};
use serde_json::Value;
use std::sync::Arc;

pub fn url_join(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("url_join", hlua::function2(move |base: String, update: String| -> Result<String> {
        let base = Url::parse(&base)
            .map_err(|err| state.set_error(Error::from(err)))?;
        let url = base.join(&update)
            .map_err(|err| state.set_error(Error::from(err)))?;

        Ok(url.into())
    }))
}

pub fn url_parse(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("url_parse", hlua::function1(move |url: String| -> Result<AnyLuaValue> {
        let url = Url::parse(&url)
            .map_err(|err| state.set_error(Error::from(err)))?;

        let mut map = LuaMap::new();

        map.insert_str("scheme", url.scheme());

        if let Some(host) = url.host_str() {
            map.insert_str("host", host);
        }

        if let Some(port) = url.port() {
            map.insert_num("port", port.into());
        }

        map.insert_str("path", url.path());

        if let Some(query) = url.query() {
            map.insert_str("query", query);
        }

        let mut query_pairs = url.query_pairs().peekable();
        if query_pairs.peek().is_some() {
            let mut params = LuaMap::new();
            for (key, value) in query_pairs {
                params.insert_str(key, value);
            }
            map.insert("params", params);
        }

        if let Some(fragment) = url.fragment() {
            map.insert_str("fragment", fragment);
        }

        Ok(map.into())
    }))
}

pub fn url_encode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("url_encode", hlua::function1(move |v: AnyLuaValue| -> Result<String> {
        let v: LuaJsonValue = v.into();
        let v: Value = v.into();
        serde_urlencoded::to_string(v)
            .map_err(|err| state.set_error(Error::from(err)))
    }))
}

pub fn url_decode(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("url_decode", hlua::function1(move |v: String| -> Result<AnyLuaValue> {
        let v: Value = serde_urlencoded::from_str(&v)
            .map_err(|err| state.set_error(Error::from(err)))?;
        let v: LuaJsonValue = v.into();
        Ok(v.into())
    }))
}

pub fn url_escape(lua: &mut hlua::Lua, _state: Arc<dyn State>) {
    lua.set("url_escape", hlua::function1(move |v: String| -> String {
        percent_encode(v.as_bytes(), NON_ALPHANUMERIC)
            .to_string()
    }))
}

pub fn url_unescape(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("url_unescape", hlua::function1(move |v: String| -> Result<String> {
        percent_decode(v.as_bytes())
            .decode_utf8()
            .map_err(|err| state.set_error(Error::from(err)))
            .map(String::from)
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_relative_path() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/abc", "bar")
            print(url)
            if url ~= "https://example.com/foo/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_relative_query() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/", "?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_relative_path_query() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/bar", "abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_absolute_path() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/abc", "/bar")
            print(url)
            if url ~= "https://example.com/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_absolute_query() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/", "/?x=1&a=2")
            print(url)
            if url ~= "https://example.com/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_absolute_path_query() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/abc", "/abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_replace() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("http://example.com/foo/?fizz=buzz", "https://asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_protocol_relative() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_join("https://example.com/foo/?fizz=buzz", "//asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_parse() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_parse("https://example.com")
            print(url)
            if url['scheme'] ~= "https" then return 'scheme' end
            if url['host'] ~= "example.com" then return 'host' end
            if url['port'] ~= nil then return 'port' end
            if url['path'] ~= "/" then return 'path' end
            if url['query'] ~= nil then return 'query' end
            if url['fragment'] ~= nil then return 'fragment' end
            if url['params'] ~= nil then return 'params' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_parse_advanced() {
        let script = Script::load_unchecked(r#"
        function run()
            url = url_parse("https://example.com:1337/foo/abc?a=b&x=1&x=2&y[]=asdf#foo")
            print(url)
            if url['scheme'] ~= "https" then return 'scheme' end
            if url['host'] ~= "example.com" then return 'host' end
            if url['port'] ~= 1337 then return 'port' end
            if url['path'] ~= "/foo/abc" then return 'path' end
            if url['query'] ~= "a=b&x=1&x=2&y[]=asdf" then return 'query' end
            if url['fragment'] ~= "foo" then return 'fragment' end
            if url['params']['a'] ~= "b" then return 'params' end
            if url['params']['x'] ~= "2" then return 'params' end
            if url['params']['y[]'] ~= "asdf" then return 'params' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_encode() {
        let script = Script::load_unchecked(r#"
        function run()
            v = url_encode({
                a='b',
                c='d',
            })
            print(v)
            if v ~= 'a=b&c=d' then return 'wrong value' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_decode() {
        let script = Script::load_unchecked(r#"
        function run()
            v = url_decode('a=b&c=d')
            print(v)
            if v['a'] ~= 'b' then return 'wrong a value' end
            if v['c'] ~= 'd' then return 'wrong c value' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_escape() {
        let script = Script::load_unchecked(r#"
        function run()
            v = url_escape('foo bar?')
            print(v)
            if v ~= 'foo%20bar%3F' then return 'wrong value' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_url_unescape() {
        let script = Script::load_unchecked(r#"
        function run()
            v = url_unescape('foo%20bar%3F')
            print(v)
            if v ~= 'foo bar?' then return 'wrong value' end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
