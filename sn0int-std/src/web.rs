pub use chrootable_https::{Client, HttpClient, Resolver, Response};

use crate::blobs::{Blob, BlobState};
use crate::engine::structs::LuaMap;
use crate::errors::*;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use base64;
use chrootable_https::http::uri::Parts;
use chrootable_https::{Body, Request, Uri};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;
use serde_json;
use serde_urlencoded;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

pub fn url_set_qs<S: Serialize + fmt::Debug>(url: Uri, query: &S) -> Result<Uri> {
    let mut parts = Parts::from(url);

    let query = serde_urlencoded::to_string(query)?;

    parts.path_and_query = Some(
        match parts.path_and_query {
            Some(pq) => format!("{}?{}", pq.path(), query),
            None => format!("/?{}", query),
        }
        .parse()?,
    );

    Uri::from_parts(parts).map_err(Error::from)
}

pub trait WebState {
    fn http(&self, proxy: &Option<SocketAddr>) -> Result<Arc<chrootable_https::Client<Resolver>>>;

    fn register_in_jar(&self, session: &str, key: String, value: String);
}

#[derive(Debug)]
pub struct HttpSession {
    id: String,
    pub cookies: CookieJar,
}

impl HttpSession {
    pub fn new() -> (String, HttpSession) {
        let id: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        (
            id.clone(),
            HttpSession {
                id,
                cookies: CookieJar::default(),
            },
        )
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct RequestOptions {
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    json: Option<serde_json::Value>,
    form: Option<serde_json::Value>,
    body: Option<String>,
    timeout: Option<u64>,
    #[serde(default)]
    into_blob: bool,
    proxy: Option<SocketAddr>,
    #[serde(default)]
    binary: bool,
}

impl RequestOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<RequestOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

pub fn default_user_agent() -> String {
    format!("sn0int/{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HttpRequest {
    // reference to the HttpSession
    session: String,
    cookies: CookieJar,
    method: String,
    url: String,
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    body: Option<ReqBody>,
    timeout: Option<Duration>,
    into_blob: bool,
    proxy: Option<SocketAddr>,
    binary: bool,
}

impl HttpRequest {
    pub fn new(
        session: &HttpSession,
        method: String,
        url: String,
        options: RequestOptions,
    ) -> HttpRequest {
        let cookies = session.cookies.clone();

        let user_agent = options.user_agent.or_else(|| Some(default_user_agent()));
        let timeout = options.timeout.map(Duration::from_millis);

        let mut request = HttpRequest {
            session: session.id.clone(),
            cookies,
            method,
            url,
            query: options.query,
            headers: options.headers,
            basic_auth: options.basic_auth,
            user_agent,
            body: None,
            timeout,
            into_blob: options.into_blob,
            proxy: options.proxy,
            binary: options.binary,
        };

        if let Some(json) = options.json {
            request.body = Some(ReqBody::Json(json));
        }

        if let Some(form) = options.form {
            request.body = Some(ReqBody::Form(form));
        }

        if let Some(text) = options.body {
            request.body = Some(ReqBody::Raw(text));
        }

        request
    }

    pub fn send(&self, state: &dyn WebState) -> Result<Response> {
        let mut url = self.url.parse::<Uri>()?;

        // set query string
        if let Some(query) = &self.query {
            url = url_set_qs(url, query)?;
        }

        // start setting up request
        let mut req = Request::builder();
        req.method(self.method.as_str());
        req.uri(url);

        let mut observed_headers = HashSet::new();

        // set cookies
        {
            use chrootable_https::header::COOKIE;
            let mut cookies = String::new();

            for (key, value) in self.cookies.iter() {
                if !cookies.is_empty() {
                    cookies += "; ";
                }
                cookies.push_str(&format!("{}={}", key, value));
            }

            if !cookies.is_empty() {
                req.header(COOKIE, cookies.as_str());
                observed_headers.insert(COOKIE.as_str().to_lowercase());
            }
        }

        // add headers
        if let Some(ref agent) = self.user_agent {
            use chrootable_https::header::USER_AGENT;
            req.header(USER_AGENT, agent.as_str());
            observed_headers.insert(USER_AGENT.as_str().to_lowercase());
        }

        if let Some(ref auth) = self.basic_auth {
            use chrootable_https::header::AUTHORIZATION;
            let &(ref user, ref password) = auth;

            let auth = base64::encode(&format!("{}:{}", user, password));
            let auth = format!("Basic {}", auth);
            req.header(AUTHORIZATION, auth.as_str());
            observed_headers.insert(AUTHORIZATION.as_str().to_lowercase());
        }

        if let Some(ref headers) = self.headers {
            for (k, v) in headers {
                req.header(k.as_str(), v.as_str());
                observed_headers.insert(k.to_lowercase());
            }
        }

        // finalize request
        let body = match self.body {
            Some(ReqBody::Raw(ref x)) => Body::from(x.clone()),
            Some(ReqBody::Form(ref x)) => {
                // if Content-Type is not set, set header
                if !observed_headers.contains("content-type") {
                    req.header("Content-Type", "application/x-www-form-urlencoded");
                }
                Body::from(serde_urlencoded::to_string(x)?)
            }
            Some(ReqBody::Json(ref x)) => {
                // if Content-Type is not set, set header
                if !observed_headers.contains("content-type") {
                    req.header("Content-Type", "application/json");
                }
                Body::from(serde_json::to_string(x)?)
            }
            None => Body::empty(),
        };
        let req = req.body(body)?;

        debug!("Getting http client");
        let client = state.http(&self.proxy)?;

        // send request
        debug!("Sending http request: {:?}", req);
        let res = client
            .request(req)
            .with_timeout(self.timeout)
            .wait_for_response()?;

        for cookie in &res.cookies {
            HttpRequest::register_cookies_on_state(&self.session, state, cookie);
        }

        Ok(res)
    }

    pub fn response_to_lua<S>(&self, state: &S, res: Response) -> Result<LuaMap>
    where
        S: WebState + BlobState,
    {
        // map result to LuaMap
        let mut resp = LuaMap::new();
        resp.insert_num("status", f64::from(res.status));

        let mut headers = LuaMap::new();
        for (key, value) in res.headers {
            headers.insert_str(key.to_lowercase(), value);
        }
        resp.insert("headers", headers);

        if let Some(ipaddr) = res.ipaddr {
            resp.insert_str("ipaddr", ipaddr.to_string());
        }

        if self.into_blob {
            let blob = Blob::create(res.body);
            let id = state.register_blob(blob);
            resp.insert_str("blob", id);
        } else if self.binary {
            resp.insert_serde("binary", &res.body[..])?;
        } else {
            resp.insert_str("text", String::from_utf8_lossy(&res.body));
        }

        Ok(resp)
    }

    fn register_cookies_on_state(session: &str, state: &dyn WebState, cookie: &str) {
        let mut key = String::new();
        let mut value = String::new();
        let mut in_key = true;

        for c in cookie.as_bytes() {
            match *c as char {
                '=' if in_key => in_key = false,
                ';' => break,
                c if in_key => key.push(c),
                c => value.push(c),
            }
        }

        state.register_in_jar(session, key, value);
    }
}

impl HttpRequest {
    pub fn try_from(x: AnyLuaValue) -> Result<HttpRequest> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl Into<AnyLuaValue> for HttpRequest {
    fn into(self) -> AnyLuaValue {
        let v = serde_json::to_value(&self).unwrap();
        LuaJsonValue::from(v).into()
    }
}

// see https://github.com/seanmonstar/reqwest/issues/14 for proper cookie jars
// maybe change this to reqwest::header::Cookie
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CookieJar(HashMap<String, String>);

impl CookieJar {
    #[inline(always)]
    pub fn register_in_jar(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }
}

impl Deref for CookieJar {
    type Target = HashMap<String, String>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReqBody {
    Raw(String), // TODO: maybe Vec<u8>
    Form(serde_json::Value),
    Json(serde_json::Value),
}
