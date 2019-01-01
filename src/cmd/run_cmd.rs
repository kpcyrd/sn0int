use crate::errors::*;

use crate::args;
use crate::db::{Database, Filter};
use crate::engine::Module;
use crate::models::*;
use crate::shell::Readline;
use crate::keyring::KeyRing;
use crate::term;
use crate::utils;
use crate::worker;
use serde::Serialize;
use serde_json;
use sn0int_common::metadata::Source;
use std::collections::HashMap;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short="j", long="threads", default_value="1")]
    threads: usize,
    #[structopt(short="v", long="verbose", parse(from_occurrences))]
    verbose: u64,
}

#[derive(Debug, Clone)]
pub struct Params<'a> {
    pub threads: usize,
    pub verbose: u64,
    pub stdin: bool,
    pub grants: &'a [String],
    pub grant_full_keyring: bool,
    pub deny_keyring: bool,
    pub exit_on_error: bool,
}

impl<'a> From<&'a args::Run> for Params<'a> {
    fn from(args: &args::Run) -> Params {
        Params {
            threads: args.threads,
            verbose: args.verbose,
            stdin: args.stdin,
            grants: &args.grants,
            grant_full_keyring: args.grant_full_keyring,
            deny_keyring: args.deny_keyring,
            exit_on_error: args.exit_on_error,
        }
    }
}

impl From<Args> for Params<'static> {
    fn from(args: Args) -> Params<'static> {
        Params {
            threads: args.threads,
            verbose: args.verbose,
            stdin: false,
            grants: &[],
            grant_full_keyring: false,
            deny_keyring: false,
            exit_on_error: false,
        }
    }
}

fn prepare_arg<T: Serialize + Model>(x: T) -> Result<(serde_json::Value, Option<String>)> {
    let pretty = x.to_string();
    let arg = serde_json::to_value(x)?;
    Ok((arg, Some(pretty)))
}

fn prepare_args<T: Scopable + Serialize + Model>(db: &Database, filter: &Filter) -> Result<Vec<(serde_json::Value, Option<String>)>> {
    db.filter::<T>(filter)?
        .into_iter()
        .map(prepare_arg)
        .collect()
}

fn prepare_keyring(keyring: &mut KeyRing, module: &Module, params: &Params) -> Result<()> {
    for namespace in keyring.unauthorized_namespaces(&module) {
        let grant_access = if params.deny_keyring {
            false
        } else if params.grant_full_keyring || params.grants.contains(namespace) {
            true
        } else {
            let msg = format!("Grant access to {:?} credentials?", namespace);
            utils::no_else_yes(&msg)?
        };

        if grant_access {
            keyring.grant_access(&module, namespace.to_string());
            term::info(&format!("Granted access to {:?}", namespace));
        }
    }

    keyring.save()
        .context("Failed to write keyring")?;

    Ok(())
}

pub fn execute(rl: &mut Readline, params: Params, options: HashMap<String, String>) -> Result<()> {
    let module = rl.module()
        .map(|m| m.to_owned())
        .ok_or_else(|| format_err!("No module selected"))?;

    prepare_keyring(rl.keyring_mut(), &module, &params)?;

    let filter = rl.scoped_targets();

    let args = match module.source() {
        Some(Source::Domains) => prepare_args::<Domain>(rl.db(), &filter),
        Some(Source::Subdomains) => prepare_args::<Subdomain>(rl.db(), &filter),
        Some(Source::IpAddrs) => prepare_args::<IpAddr>(rl.db(), &filter),
        Some(Source::Urls) => prepare_args::<Url>(rl.db(), &filter),
        Some(Source::Emails) => prepare_args::<Email>(rl.db(), &filter),
        Some(Source::PhoneNumbers) => prepare_args::<PhoneNumber>(rl.db(), &filter),
        Some(Source::KeyRing(namespace)) => {
            let keyring = rl.keyring();
            if keyring.is_access_granted(&module, &namespace) {
                keyring.get_all_for(&namespace).into_iter()
                    .map(|key| {
                        let pretty = format!("{}:{}", key.namespace, key.access_key);
                        let arg = serde_json::to_value(key)?;
                        Ok((arg, Some(pretty)))
                    })
                    .collect::<Result<Vec<_>>>()
            } else {
                Ok(vec![])
            }
        },
        None => Ok(vec![(serde_json::Value::Null, None)]),
    }?;

    rl.signal_register().catch_ctrl();
    let errors = worker::spawn(rl, &module, args, &params, rl.config().network.proxy.clone(), options);
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

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    let options = match rl.options_mut() {
        Some(options) => options.clone(),
        _ => HashMap::new(),
    };
    execute(rl, args.into(), options)
}
