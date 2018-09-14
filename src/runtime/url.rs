use errors::*;
use engine::ctx::State;
use engine::structs::LuaMap;
/*
use dns::DnsResolver;
use web::HttpClient;
*/
use hlua::{self, AnyLuaValue};
use url::Url;
use std::sync::Arc;

pub fn url_join(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("url_join", hlua::function2(move |base: String, update: String| -> Result<String> {
        let base = Url::parse(&base)
            .map_err(|err| state.set_error(Error::from(err)))?;
        let url = base.join(&update)
            .map_err(|err| state.set_error(Error::from(err)))?;

        Ok(url.into_string())
    }))
}

pub fn url_parse(lua: &mut hlua::Lua, state: Arc<State>) {
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

/*
#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    fn verify_relative_path() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_path"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "bar")
            print(url)
            if url ~= "https://example.com/foo/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_relative_query() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/", "?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_relative_path_query() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_path_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/bar", "abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_path() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_path"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "/bar")
            print(url)
            if url ~= "https://example.com/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_query() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/", "/?x=1&a=2")
            print(url)
            if url ~= "https://example.com/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_path_query() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_path_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "/abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_replace() {
        let script = Loader::init_default(r#"
        descr = "verify_replace"

        function detect() end
        function decap()
            url = url_join("http://example.com/foo/?fizz=buzz", "https://asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_protocol_relative() {
        let script = Loader::init_default(r#"
        descr = "verify_protocol_relative"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/?fizz=buzz", "//asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_url_parse() {
        let script = Loader::init_default(r#"
        descr = "verify_url_parse"

        function detect() end
        function decap()
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
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_url_parse_advanced() {
        let script = Loader::init_default(r#"
        descr = "verify_url_parse_advanced"

        function detect() end
        function decap()
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
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
*/
