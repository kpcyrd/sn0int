use crate::errors::*;

use crate::keyring::{KeyName, KeyRing};
use crate::shell::Shell;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::utils;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub enum Args {
    #[structopt(name="add")]
    /// Add a new key to the keyring
    Add(KeyRingAdd),
    #[structopt(name="delete")]
    /// Delete a key from the keyring
    Delete(KeyRingDelete),
    #[structopt(name="get")]
    /// Get a key from the keyring
    Get(KeyRingGet),
    #[structopt(name="list")]
    /// List keys in the keyring
    List(KeyRingList),
}

#[derive(Debug, StructOpt)]
pub struct KeyRingAdd {
    key: KeyName,
    secret: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct KeyRingDelete {
    key: KeyName,
}

#[derive(Debug, StructOpt)]
pub struct KeyRingGet {
    key: KeyName,
    #[structopt(short="q",
                long="quiet")]
    /// Only output secret key
    quiet: bool,
}

#[derive(Debug, StructOpt)]
pub struct KeyRingList {
    namespace: Option<String>,
}

pub fn run(rl: &mut Shell, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Add(add) => keyring_add(rl, add),
        Args::Delete(delete) => keyring_delete(rl, delete),
        Args::Get(get) => keyring_get(rl.keyring(), &get),
        Args::List(list) => keyring_list(rl.keyring(), list),
    }
}

fn keyring_add(rl: &mut Shell, add: KeyRingAdd) -> Result<()> {
    let keyring = rl.keyring_mut();

    // TODO: there's no non-interactive way to add a key without a secret key
    let secret = match add.secret {
        Some(secret) => Some(secret),
        None => utils::question_opt("Secretkey")?,
    };

    keyring.insert(add.key, secret)?;
    rl.reload_keyring_cache();
    Ok(())
}

fn keyring_delete(rl: &mut Shell, delete: KeyRingDelete) -> Result<()> {
    let keyring = rl.keyring_mut();
    keyring.delete(delete.key)?;
    rl.reload_keyring_cache();
    Ok(())
}

fn keyring_get(keyring: &KeyRing, get: &KeyRingGet) -> Result<()> {
    if let Some(key) = keyring.get(&get.key) {
        if get.quiet {
            if let Some(secret_key) = key.secret_key {
                println!("{}", secret_key);
            }
        } else {
            println!("Namespace:    {:?}", get.key.namespace);
            println!("Access Key:   {:?}", get.key.name);
            if let Some(secret_key) = key.secret_key {
                println!("Secret:       {:?}", secret_key);
            }
        }
    }
    Ok(())
}

fn keyring_list(keyring: &KeyRing, list: KeyRingList) -> Result<()> {
    let list = match list.namespace {
        Some(namespace) => keyring.list_for(&namespace),
        None => keyring.list(),
    };

    for key in list {
        println!("{}:{}", key.namespace, key.name);
    }

    Ok(())
}
