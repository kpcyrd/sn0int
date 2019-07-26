use crate::errors::*;

use crate::blobs::Blob;
use crate::db::Family;
use crate::engine::{Environment, Reporter};
use crate::geoip::{MaxmindReader, GeoIP, AsnDB};
use crate::hlua::{self, AnyLuaValue};
use crate::keyring::KeyRingEntry;
use crate::models::{Insert, Update};
use crate::psl::{Psl, PslReader};
use crate::lazy::Lazy;
use crate::runtime;
use crate::sockets::{Socket, SocketOptions, TlsData};
use crate::web::{HttpSession, HttpRequest, RequestOptions};
use crate::worker::{Event, LogEvent, DatabaseEvent, StdioEvent};
use chrootable_https::{self, Resolver};
use serde_json;
use std::collections::HashMap;
use std::result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use rand::prelude::*;
use rand::distributions::Alphanumeric;


pub trait State {
    fn clear_error(&self);

    fn last_error(&self) -> Option<String>;

    fn set_error(&self, err: Error) -> Error;

    fn send(&self, msg: &Event);

    fn recv(&self) -> Result<serde_json::Value>;

    fn verbose(&self) -> u64;

    #[inline]
    fn info(&self, msg: String) {
        self.send(&Event::Log(LogEvent::Info(msg)))
    }

    #[inline]
    fn debug(&self, msg: String) {
        if self.verbose() >= 2 {
            self.send(&Event::Log(LogEvent::Debug(msg)))
        }
    }

    #[inline]
    fn error(&self, msg: String) {
        self.send(&Event::Log(LogEvent::Error(msg)))
    }

    #[inline]
    fn warn(&self, msg: String) {
        self.send(&Event::Log(LogEvent::Warn(msg)))
    }

    #[inline]
    fn warn_once(&self, msg: String) {
        self.send(&Event::Log(LogEvent::WarnOnce(msg)))
    }

    #[inline]
    fn status(&self, msg: String) {
        self.send(&Event::Log(LogEvent::Status(msg)))
    }

    fn db_insert(&self, object: Insert) -> Result<Option<i32>> {
        self.send(&Event::Database(DatabaseEvent::Insert(object)));
        let reply = self.recv()?;
        let reply: result::Result<Option<i32>, String> = serde_json::from_value(reply)?;

        reply.map_err(|err| format_err!("Failed to add to database: {:?}", err))
    }

    fn db_insert_ttl(&self, object: Insert, ttl: i32) -> Result<Option<i32>> {
        self.send(&Event::Database(DatabaseEvent::InsertTtl((object, ttl))));
        let reply = self.recv()?;
        let reply: result::Result<Option<i32>, String> = serde_json::from_value(reply)?;

        reply.map_err(|err| format_err!("Failed to add to database: {:?}", err))
    }

    fn db_select(&self, family: Family, value: String) -> Result<Option<i32>> {
        self.send(&Event::Database(DatabaseEvent::Select((family, value))));
        let reply = self.recv()?;
        let reply: result::Result<Option<i32>, String> = serde_json::from_value(reply)?;

        reply.map_err(|err| format_err!("Failed to query database: {:?}", err))
    }

    fn db_update(&self, object: String, update: Update) -> Result<Option<i32>> {
        self.send(&Event::Database(DatabaseEvent::Update((object, update))));
        let reply = self.recv()?;
        let reply: result::Result<Option<i32>, String> = serde_json::from_value(reply)?;

        reply.map_err(|err| format_err!("Failed to update database: {:?}", err))
    }

    fn stdin_readline(&self) -> Result<Option<String>> {
        self.send(&Event::Stdio(StdioEvent {}));
        let reply = self.recv()?;
        let reply: result::Result<Option<String>, String> = serde_json::from_value(reply)?;
        reply.map_err(|err| format_err!("Failed to read stdin: {:?}", err))
    }

    #[inline]
    fn random_id(&self) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(16).collect()
    }

    fn keyring(&self, namespace: &str) -> Vec<&KeyRingEntry>;

    fn dns_config(&self) -> &Resolver;

    fn proxy(&self) -> Option<&SocketAddr>;

    fn getopt(&self, key: &str) -> Option<&String>;

    fn psl(&self) -> Result<Arc<Psl>>;

    fn geoip(&self) -> Result<Arc<GeoIP>>;

    fn asn(&self) -> Result<Arc<AsnDB>>;

    fn sock_connect(&self, host: &str, port: u16, options: &SocketOptions) -> Result<String>;

    fn get_sock(&self, id: &str)-> Arc<Mutex<Socket>>;

    fn sock_upgrade_tls(&self, id: &str, options: &SocketOptions) -> Result<TlsData>;

    fn http(&self) -> &chrootable_https::Client<Resolver>;

    fn http_mksession(&self) -> String;

    fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest;

    fn register_in_jar(&self, session: &str, key: String, value: String);

    fn register_blob(&self, blob: Blob) -> String;

    fn get_blob(&self, id: &str) -> Result<Arc<Blob>>;

    fn persist_blob(&self, id: &str) -> Result<()> {
        let blob = self.get_blob(id)?;
        self.send(&Event::Blob(blob.as_ref().clone()));
        let reply = self.recv()?;
        let reply: result::Result<(), String> = serde_json::from_value(reply)?;
        reply.map_err(|err| format_err!("Failed to store blob: {:?}", err))
    }
}

// #[derive(Debug)]
pub struct LuaState {
    error: Mutex<Option<Error>>,
    logger: Arc<Mutex<Box<Reporter>>>,
    socket_sessions: Mutex<HashMap<String, Arc<Mutex<Socket>>>>,
    blobs: Mutex<HashMap<String, Arc<Blob>>>,
    http_sessions: Mutex<HashMap<String, HttpSession>>,
    http: chrootable_https::Client<Resolver>,

    verbose: u64,
    keyring: Vec<KeyRingEntry>, // TODO: maybe hashmap
    dns_config: Resolver,
    psl: Mutex<Lazy<PslReader, Arc<Psl>>>,
    geoip: Mutex<Lazy<MaxmindReader, Arc<GeoIP>>>,
    asn: Mutex<Lazy<MaxmindReader, Arc<AsnDB>>>,
    proxy: Option<SocketAddr>,
    options: HashMap<String, String>,
}

impl State for LuaState {
    fn clear_error(&self) {
        let mut mtx = self.error.lock().unwrap();
        mtx.take();
    }

    fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        lock.as_ref().map(|err| err.to_string())
    }

    fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = format_err!("{:?}", err);
        *mtx = Some(err);
        cp
    }

    fn send(&self, msg: &Event) {
        let mut tx = self.logger.lock().unwrap();
        tx.send(msg).expect("Failed to write event");
    }

    fn recv(&self) -> Result<serde_json::Value> {
        let mut tx = self.logger.lock().unwrap();
        tx.recv()
    }

    fn verbose(&self) -> u64 {
        self.verbose
    }

    fn keyring(&self, query: &str) -> Vec<&KeyRingEntry> {
        self.keyring.iter()
            .filter(|x| x.matches(query))
            .collect()
    }

    fn dns_config(&self) -> &Resolver {
        &self.dns_config
    }

    fn proxy(&self) -> Option<&SocketAddr> {
        self.proxy.as_ref()
    }

    fn getopt(&self, key: &str) -> Option<&String> {
        self.options.get(key)
    }

    fn psl(&self) -> Result<Arc<Psl>> {
        let mut psl = self.psl.lock().unwrap();
        let psl = psl.get()?;
        Ok(psl.clone())
    }

    fn geoip(&self) -> Result<Arc<GeoIP>> {
        let mut geoip = self.geoip.lock().unwrap();
        let geoip = geoip.get()?;
        Ok(geoip.clone())
    }

    fn asn(&self) -> Result<Arc<AsnDB>> {
        let mut asn = self.asn.lock().unwrap();
        let asn = asn.get()?;
        Ok(asn.clone())
    }

    fn sock_connect(&self, host: &str, port: u16, options: &SocketOptions) -> Result<String> {
        let mut mtx = self.socket_sessions.lock().unwrap();
        let id = self.random_id();

        let sock = match &self.proxy {
            Some(proxy) => Socket::connect_socks5(proxy, host, port, options)?,
            _ => Socket::connect(&self.dns_config, host, port, options)?,
        };

        mtx.insert(id.clone(), Arc::new(Mutex::new(sock)));

        Ok(id)
    }

    fn get_sock(&self, id: &str)-> Arc<Mutex<Socket>> {
        let mtx = self.socket_sessions.lock().unwrap();
        let sock = mtx.get(id).expect("invalid session reference"); // TODO
        sock.clone()
    }

    fn sock_upgrade_tls(&self, id: &str, options: &SocketOptions) -> Result<TlsData> {
        let mut mtx = self.socket_sessions.lock().unwrap();
        let sock = mtx.remove(id).expect("invalid session reference"); // TODO

        let sock = Arc::try_unwrap(sock).unwrap();
        let sock = sock.into_inner().unwrap();

        let (sock, tls) = sock.upgrade_to_tls(options)?;

        mtx.insert(id.to_string(), Arc::new(Mutex::new(sock)));

        Ok(tls)
    }

    fn http(&self) -> &chrootable_https::Client<Resolver> {
        &self.http
    }

    fn http_mksession(&self) -> String {
        let mut mtx = self.http_sessions.lock().unwrap();
        let (id, session) = HttpSession::new();
        mtx.insert(id.clone(), session);
        id
    }

    fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let mtx = self.http_sessions.lock().unwrap();
        let session = mtx.get(session_id).expect("invalid session reference"); // TODO

        HttpRequest::new(&session, method, url, options)
    }

    fn register_in_jar(&self, session: &str, key: String, value: String) {
        let mut mtx = self.http_sessions.lock().unwrap();
        if let Some(session) = mtx.get_mut(session) {
            session.cookies.register_in_jar(key, value);
        }
    }

    fn register_blob(&self, blob: Blob) -> String {
        let id = blob.id.clone();

        let mut mtx = self.blobs.lock().unwrap();
        mtx.insert(id.clone(), Arc::new(blob));
        debug!("Registered blob: {:?}", id);

        id
    }

    fn get_blob(&self, id: &str) -> Result<Arc<Blob>> {
        let mtx = self.blobs.lock().unwrap();
        let blob = mtx.get(id)
            .ok_or_else(|| format_err!("Invalid blob reference"))?;
        Ok(blob.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    code: String,
}

fn ctx<'a>(env: Environment, logger: Arc<Mutex<Box<Reporter>>>) -> (hlua::Lua<'a>, Arc<LuaState>) {
    debug!("Creating lua context");
    let mut lua = hlua::Lua::new();
    lua.open_string();

    let http = match env.proxy {
        Some(proxy) => chrootable_https::Client::with_socks5(proxy),
        _ => {
            let resolver = env.dns_config.clone();
            chrootable_https::Client::new(resolver)
        },
    };

    let state = Arc::new(LuaState {
        error: Mutex::new(None),
        logger,
        socket_sessions: Mutex::new(HashMap::new()),
        blobs: Mutex::new(HashMap::new()),
        http_sessions: Mutex::new(HashMap::new()),
        http,

        verbose: env.verbose,
        keyring: env.keyring,
        dns_config: env.dns_config,
        psl: Mutex::new(Lazy::from(env.psl)),
        geoip: Mutex::new(Lazy::from(env.geoip)),
        asn: Mutex::new(Lazy::from(env.asn)),
        proxy: env.proxy,
        options: env.options,
    });

    debug!("Adding all blobs from StartCommand");
    for blob in env.blobs {
        state.register_blob(blob);
    }

    runtime::asn_lookup(&mut lua, state.clone());
    runtime::base64_decode(&mut lua, state.clone());
    runtime::base64_encode(&mut lua, state.clone());
    runtime::base64_custom_decode(&mut lua, state.clone());
    runtime::base64_custom_encode(&mut lua, state.clone());
    runtime::base32_custom_decode(&mut lua, state.clone());
    runtime::base32_custom_encode(&mut lua, state.clone());
    runtime::clear_err(&mut lua, state.clone());
    runtime::create_blob(&mut lua, state.clone());
    runtime::datetime(&mut lua, state.clone());
    runtime::db_add(&mut lua, state.clone());
    runtime::db_add_ttl(&mut lua, state.clone());
    runtime::db_select(&mut lua, state.clone());
    runtime::db_update(&mut lua, state.clone());
    runtime::debug(&mut lua, state.clone());
    runtime::dns(&mut lua, state.clone());
    runtime::error(&mut lua, state.clone());
    runtime::geoip_lookup(&mut lua, state.clone());
    runtime::getopt(&mut lua, state.clone());
    runtime::hex(&mut lua, state.clone());
    runtime::hmac_md5(&mut lua, state.clone());
    runtime::hmac_sha1(&mut lua, state.clone());
    runtime::hmac_sha2_256(&mut lua, state.clone());
    runtime::hmac_sha2_512(&mut lua, state.clone());
    runtime::hmac_sha3_256(&mut lua, state.clone());
    runtime::hmac_sha3_512(&mut lua, state.clone());
    runtime::html_select(&mut lua, state.clone());
    runtime::html_select_list(&mut lua, state.clone());
    runtime::http_mksession(&mut lua, state.clone());
    runtime::http_request(&mut lua, state.clone());
    runtime::http_send(&mut lua, state.clone());
    runtime::http_fetch_json(&mut lua, state.clone());
    runtime::img_exif(&mut lua, state.clone());
    runtime::img_load(&mut lua, state.clone());
    runtime::img_nudity(&mut lua, state.clone());
    runtime::info(&mut lua, state.clone());
    runtime::json_decode(&mut lua, state.clone());
    runtime::json_decode_stream(&mut lua, state.clone());
    runtime::json_encode(&mut lua, state.clone());
    runtime::keyring(&mut lua, state.clone());
    runtime::last_err(&mut lua, state.clone());
    runtime::md5(&mut lua, state.clone());
    runtime::pgp_pubkey(&mut lua, state.clone());
    runtime::pgp_pubkey_armored(&mut lua, state.clone());
    runtime::print(&mut lua, state.clone());
    runtime::psl_domain_from_dns_name(&mut lua, state.clone());
    runtime::regex_find(&mut lua, state.clone());
    runtime::regex_find_all(&mut lua, state.clone());
    runtime::semver_match(&mut lua, state.clone());
    runtime::set_err(&mut lua, state.clone());
    runtime::sha1(&mut lua, state.clone());
    runtime::sha2_256(&mut lua, state.clone());
    runtime::sha2_512(&mut lua, state.clone());
    runtime::sha3_256(&mut lua, state.clone());
    runtime::sha3_512(&mut lua, state.clone());
    runtime::sleep(&mut lua, state.clone());
    runtime::sn0int_version(&mut lua, state.clone());
    runtime::sock_connect(&mut lua, state.clone());
    runtime::sock_upgrade_tls(&mut lua, state.clone());
    runtime::sock_send(&mut lua, state.clone());
    runtime::sock_recv(&mut lua, state.clone());
    runtime::sock_sendline(&mut lua, state.clone());
    runtime::sock_recvline(&mut lua, state.clone());
    runtime::sock_recvall(&mut lua, state.clone());
    runtime::sock_recvline_contains(&mut lua, state.clone());
    runtime::sock_recvline_regex(&mut lua, state.clone());
    runtime::sock_recvn(&mut lua, state.clone());
    runtime::sock_recvuntil(&mut lua, state.clone());
    runtime::sock_sendafter(&mut lua, state.clone());
    runtime::sock_newline(&mut lua, state.clone());
    runtime::status(&mut lua, state.clone());
    runtime::stdin_readline(&mut lua, state.clone());
    runtime::strftime(&mut lua, state.clone());
    runtime::strptime(&mut lua, state.clone());
    runtime::time_unix(&mut lua, state.clone());
    runtime::url_decode(&mut lua, state.clone());
    runtime::url_encode(&mut lua, state.clone());
    runtime::url_escape(&mut lua, state.clone());
    runtime::url_join(&mut lua, state.clone());
    runtime::url_parse(&mut lua, state.clone());
    runtime::url_unescape(&mut lua, state.clone());
    runtime::utf8_decode(&mut lua, state.clone());
    runtime::warn(&mut lua, state.clone());
    runtime::warn_once(&mut lua, state.clone());
    runtime::x509_parse_pem(&mut lua, state.clone());
    runtime::xml_decode(&mut lua, state.clone());
    runtime::xml_named(&mut lua, state.clone());

    debug!("Created lua context");

    (lua, state)
}

impl Script {
    pub fn load_unchecked<I: Into<String>>(code: I) -> Result<Script> {
        /*
        let (mut lua, _) = ctx();

        // TODO: we do not want to execute the script outside of the sandbox
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: hlua::StringInLua<_> = lua.get("descr")
                .ok_or_else(|| format_err!("descr undefined"))?;
            (*descr).to_owned()
        };
        */

        Ok(Script {
            code: code.into(),
        })
    }

    pub fn run(&self, env: Environment,
                      tx: Arc<Mutex<Box<Reporter>>>,
                      arg: AnyLuaValue,
    ) -> Result<()> {
        let (mut lua, state) = ctx(env, tx);

        debug!("Initializing lua module");
        lua.execute::<()>(&self.code)?;

        let run: Result<_> = lua.get("run")
            .ok_or_else(|| format_err!( "run undefined"));
        let mut run: hlua::LuaFunction<_> = run?;

        debug!("Starting lua script");
        let result: hlua::AnyLuaValue = run.call_with_args(arg)
            .map_err(|err| format_err!("execution failed: {:?}", err))?;

        debug!("Lua script terminated");

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err);
        }

        use crate::hlua::AnyLuaValue::*;
        match result {
            LuaString(x) => bail!("Script returned error: {:?}", x),
            _ => Ok(())
        }
    }

    #[cfg(test)]
    pub fn test(&self) -> Result<()> {
        use crate::engine::tests::DummyReporter;
        use crate::geoip::Maxmind;
        let keyring = Vec::new();
        let dns_config = Resolver::from_system()?;
        let proxy = None;
        let psl = PslReader::String(r#"
// ===BEGIN ICANN DOMAINS===
com
// ===END ICANN DOMAINS===
// ===BEGIN PRIVATE DOMAINS===
a.prod.fastly.net
// ===END PRIVATE DOMAINS===
"#.into());
        let geoip = GeoIP::open_reader()?;
        let asn = AsnDB::open_reader()?;

        let env = Environment {
            verbose: 0,
            keyring,
            dns_config,
            proxy,
            options: HashMap::new(),
            blobs: Vec::new(),
            psl,
            geoip,
            asn,
        };
        self.run(env, DummyReporter::new(), AnyLuaValue::LuaNil)
    }
}
