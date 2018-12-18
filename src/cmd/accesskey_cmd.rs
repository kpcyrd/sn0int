use crate::errors::*;

use crate::accesskey::{KeyName, KeyStore};
use crate::shell::Readline;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::utils;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub enum Args {
    #[structopt(name="add")]
    /// Add a new key to the keystore
    Add(AccessKeyAdd),
    #[structopt(name="delete")]
    /// Delete a key from the keystore
    Delete(AccessKeyDelete),
    #[structopt(name="get")]
    /// Get a key from the keystore
    Get(AccessKeyGet),
    #[structopt(name="list")]
    /// List keys in the keystore
    List(AccessKeyList),
}

#[derive(Debug, StructOpt)]
pub struct AccessKeyAdd {
    key: KeyName,
    secret: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct AccessKeyDelete {
    key: KeyName,
}

#[derive(Debug, StructOpt)]
pub struct AccessKeyGet {
    key: KeyName,
    #[structopt(short="q",
                long="quiet")]
    /// Only output secret key
    quiet: bool,
}

#[derive(Debug, StructOpt)]
pub struct AccessKeyList {
    namespace: Option<String>,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Add(add) => accesskey_add(rl.keystore_mut(), add),
        Args::Delete(delete) => accesskey_delete(rl.keystore_mut(), delete),
        Args::Get(get) => accesskey_get(rl.keystore(), get),
        Args::List(list) => accesskey_list(rl.keystore(), list),
    }
}

fn accesskey_add(keystore: &mut KeyStore, add: AccessKeyAdd) -> Result<()> {
    let secret = match add.secret {
        Some(secret) => secret,
        None => utils::question("Secretkey")?,
    };

    keystore.insert(add.key, secret)
}

fn accesskey_delete(keystore: &mut KeyStore, delete: AccessKeyDelete) -> Result<()> {
    keystore.delete(delete.key)
}

fn accesskey_get(keystore: &KeyStore, get: AccessKeyGet) -> Result<()> {
    if let Some(key) = keystore.get(&get.key) {
        if get.quiet {
            println!("{}", key);
        } else {
            println!("Namespace:    {:?}", get.key.namespace);
            println!("Access Key:   {:?}", get.key.name);
            println!("Secret:       {:?}", key);
        }
    }
    Ok(())
}

fn accesskey_list(keystore: &KeyStore, list: AccessKeyList) -> Result<()> {
    let list = match list.namespace {
        Some(namespace) => keystore.list_for(&namespace),
        None => keystore.list(),
    };

    for key in list {
        println!("{}:{}", key.namespace, key.name);
    }

    Ok(())
}
