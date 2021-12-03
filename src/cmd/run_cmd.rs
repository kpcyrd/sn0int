use crate::errors::*;

use chrootable_https::dns::Resolver;
use crate::args;
use crate::blobs::{Blob, BlobStorage};
use crate::cmd::Cmd;
use crate::db::{ttl, Filter};
use crate::engine::Module;
use crate::ipc::common::StartCommand;
use crate::models::*;
use crate::shell::Shell;
use crate::keyring::KeyRing;
use crate::term;
use crate::utils;
use crate::worker;
use serde::Serialize;
use sn0int_common::metadata::Source;
use sn0int_std::ratelimits::Ratelimiter;
use std::collections::HashMap;
use std::net::SocketAddr;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Execute a module that has been installed
    pub module: Option<String>,
    /// Run investigations concurrently
    #[structopt(short="j", default_value="1")]
    pub threads: usize,
    /// Verbose logging, once to print inserts even if they don't add new
    /// data, twice to activate the debug() function
    #[structopt(short="v", long, parse(from_occurrences))]
    pub verbose: u64,
    /// Set a specific socks5 proxy to use
    #[structopt(short="X", long)]
    pub proxy: Option<SocketAddr>,
    /// Set a different default user agent
    #[structopt(long)]
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Params<'a> {
    pub module: Option<&'a String>,
    pub threads: usize,
    pub verbose: u64,
    pub stdin: bool,
    pub grants: &'a [String],
    pub grant_full_keyring: bool,
    pub deny_keyring: bool,
    pub exit_on_error: bool,
    pub proxy: Option<SocketAddr>,
    pub user_agent: Option<&'a String>,
}

impl<'a> Params<'a> {
    pub fn get_module(&self, rl: &Shell) -> Result<Module> {
        let module = if let Some(module) = self.module {
            rl.library().get(module)?
                .clone()
        } else {
            rl.module()
                .map(|m| m.to_owned())
                .ok_or_else(|| format_err!("No module selected"))?
        };
        Ok(module)
    }

    pub fn get_proxy(&self, rl: &Shell) -> Option<SocketAddr> {
        if self.proxy.is_some() {
            self.proxy
        } else {
            rl.config().network.proxy
        }
    }

    pub fn get_user_agent(&self, rl: &Shell) -> Option<String> {
        if let Some(user_agent) = self.user_agent {
            Some(user_agent.to_string())
        } else {
            rl.config().network.user_agent.clone()
        }
    }
}

impl<'a> From<&'a args::Run> for Params<'a> {
    fn from(args: &args::Run) -> Params {
        Params {
            module: args.run.module.as_ref(),
            threads: args.run.threads,
            verbose: args.run.verbose,
            stdin: args.stdin,
            grants: &args.grants,
            grant_full_keyring: args.grant_full_keyring,
            deny_keyring: args.deny_keyring,
            exit_on_error: args.exit_on_error,
            proxy: args.run.proxy,
            user_agent: args.run.user_agent.as_ref(),
        }
    }
}

impl<'a> From<&'a Args> for Params<'a> {
    fn from(args: &Args) -> Params {
        Params {
            module: args.module.as_ref(),
            threads: args.threads,
            verbose: args.verbose,
            stdin: false,
            grants: &[],
            grant_full_keyring: false,
            deny_keyring: false,
            exit_on_error: false,
            proxy: args.proxy,
            user_agent: args.user_agent.as_ref(),
        }
    }
}

fn prepare_arg<T: Serialize + Model>(bs: &BlobStorage, x: T) -> Result<(serde_json::Value, Option<String>, Vec<Blob>)> {
    let pretty = x.to_string();

    let blobs = if let Some(blob) = x.blob() {
        let blob = bs.load(blob)?;
        vec![blob]
    } else {
        Vec::new()
    };

    let arg = serde_json::to_value(x)?;
    Ok((arg, Some(pretty), blobs))
}

fn prepare_args<T: Scopable + Serialize + Model>(rl: &Shell, filter: &Filter, param: Option<&String>) -> Result<Vec<(serde_json::Value, Option<String>, Vec<Blob>)>> {
    let db = rl.db();
    let bs = rl.blobs();
    db.filter_with_param::<T>(filter, param)?
        .into_iter()
        .map(|x| prepare_arg(bs, x))
        .collect()
}

pub fn prepare_keyring(keyring: &mut KeyRing, module: &Module, params: &Params) -> Result<()> {
    for namespace in keyring.unauthorized_namespaces(module) {
        let grant_access = if params.deny_keyring {
            false
        } else if params.grant_full_keyring || params.grants.contains(namespace) {
            true
        } else {
            let msg = format!("Grant access to {:?} credentials?", namespace);
            utils::no_else_yes(&msg)?
        };

        if grant_access {
            keyring.grant_access(module, namespace.to_string());
            term::info(&format!("Granted access to {:?}", namespace));
        }
    }

    keyring.save()
        .context("Failed to write keyring")?;

    Ok(())
}

fn get_args(rl: &mut Shell, module: &Module) -> Result<Vec<(serde_json::Value, Option<String>, Vec<Blob>)>> {
    let filter = rl.scoped_targets();

    match module.source() {
        Some(Source::Domains) => prepare_args::<Domain>(rl, &filter, None),
        Some(Source::Subdomains) => prepare_args::<Subdomain>(rl, &filter, None),
        Some(Source::IpAddrs) => prepare_args::<IpAddr>(rl, &filter, None),
        Some(Source::Urls) => prepare_args::<Url>(rl, &filter, None),
        Some(Source::Emails) => prepare_args::<Email>(rl, &filter, None),
        Some(Source::PhoneNumbers) => prepare_args::<PhoneNumber>(rl, &filter, None),
        Some(Source::Networks) => prepare_args::<Network>(rl, &filter, None),
        Some(Source::Devices) => prepare_args::<Device>(rl, &filter, None),
        Some(Source::Accounts(service)) => prepare_args::<Account>(rl, &filter, service.as_ref()),
        Some(Source::Breaches) => prepare_args::<Breach>(rl, &filter, None),
        Some(Source::Images) => prepare_args::<Image>(rl, &filter, None),
        Some(Source::Ports) => prepare_args::<Port>(rl, &filter, None),
        Some(Source::Netblocks) => prepare_args::<Netblock>(rl, &filter, None),
        Some(Source::CryptoAddrs(currency)) => prepare_args::<CryptoAddr>(rl, &filter, currency.as_ref()),
        Some(Source::Notifications) => bail!("Notification modules can't be executed like this"),
        Some(Source::KeyRing(namespace)) => {
            let keyring = rl.keyring();
            if keyring.is_access_granted(module, namespace) {
                keyring.get_all_for(namespace).into_iter()
                    .map(|key| {
                        let pretty = format!("{}:{}", key.namespace, key.access_key);
                        let arg = serde_json::to_value(key)?;
                        Ok((arg, Some(pretty), vec![]))
                    })
                    .collect::<Result<Vec<_>>>()
            } else {
                Ok(vec![])
            }
        },
        None => Ok(vec![(serde_json::Value::Null, None, vec![])]),
    }
}

pub fn dump_sandbox_init_msg(rl: &mut Shell, params: Params, options: HashMap<String, String>) -> Result<()> {
    let module = params.get_module(rl)?;
    let proxy = params.get_proxy(rl);
    let user_agent = params.get_user_agent(rl);

    prepare_keyring(rl.keyring_mut(), &module, &params)?;
    let keyring = rl.keyring().request_keys(&module);

    let dns_config = Resolver::from_system_v4()?;

    let args = get_args(rl, &module)?;
    for (arg, _pretty_arg, blobs) in args {
        let start_cmd = StartCommand::new(params.verbose,
                                          keyring.clone(),
                                          dns_config.clone(),
                                          proxy,
                                          user_agent.clone(),
                                          options.clone(),
                                          module.clone(),
                                          arg,
                                          blobs);
        let out = serde_json::to_string(&start_cmd)?;
        println!("{}", out);
    }

    Ok(())
}

pub fn execute(rl: &mut Shell, params: Params, options: HashMap<String, String>) -> Result<()> {
    let module = params.get_module(rl)?;
    let proxy = params.get_proxy(rl);
    let user_agent = params.get_user_agent(rl);

    prepare_keyring(rl.keyring_mut(), &module, &params)?;
    let args = get_args(rl, &module)?;

    rl.signal_register().catch_ctrl();
    let errors = worker::spawn(rl, &module, &mut Ratelimiter::new(), args, &params, proxy, user_agent, options);
    rl.signal_register().reset_ctrlc();

    if errors > 0 {
        term::info(&format!("Finished {} ({} errors)", module.canonical(), errors));

        if params.exit_on_error {
            bail!("Some scripts failed");
        }
    } else {
        term::info(&format!("Finished {}", module.canonical()));
    }

    Ok(())
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        ttl::reap_expired(rl)?;
        let options = match rl.options_mut() {
            Some(options) => options.clone(),
            _ => HashMap::new(),
        };
        execute(rl, Params::from(&self), options)
    }
}
