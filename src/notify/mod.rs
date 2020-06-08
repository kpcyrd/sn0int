use crate::errors::*;
use crate::cmd::run_cmd::prepare_keyring;
use crate::cmd::run_cmd::Params;
use crate::options;
use crate::shell::Shell;
use crate::term;
use crate::worker;
use sn0int_common::metadata::Source;
use sn0int_std::blobs::Blob;
use std::collections::HashMap;

#[derive(Debug, StructOpt, Serialize)]
pub struct Notification {
    pub subject: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub workspaces: Vec<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub script: String,
    #[serde(default)]
    pub options: Vec<options::Opt>,
}

fn apply_rule(name: &str, filter: &[String], value: &str) -> bool {
    if !filter.is_empty() {
        debug!("{} filter is active", name);
        if !filter.iter().any(|x| x == value) {
            debug!("{} isn't allow-listed, aborting", name);
            return false;
        }
        debug!("{} was allow-listed", name);
    }
    true
}

impl NotificationConfig {
    fn matches(&self, workspace: &str, topic: &str) -> bool {
        debug!("testing notification with rules");
        if !apply_rule("workspace", &self.workspaces, workspace) {
            return false;
        }
        if !apply_rule("topic", &self.topics, topic) {
            return false;
        }
        debug!("notification matches this config");
        true
    }
}

fn prepare_arg(notification: &Notification) -> Result<(serde_json::Value, Option<String>, Vec<Blob>)> {
    let arg = serde_json::to_value(notification)?;
    Ok((arg, None, vec![]))
}

pub fn exec(rl: &mut Shell, module: &str, options: HashMap<String, String>, notification: &Notification) -> Result<usize> {
    let module = rl.library().get(&module)?.clone();

    if *module.source() != Some(Source::Notifications) {
        bail!("Module doesn't take notifications as source");
    }

    let params = Params {
        threads: 1,
        verbose: 0, // TODO: args.verbose
        stdin: false,
        grants: &[],
        grant_full_keyring: false,
        deny_keyring: false,
        exit_on_error: false,
    };

    prepare_keyring(rl.keyring_mut(), &module, &params)?;
    let args = vec![prepare_arg(&notification)?];

    rl.signal_register().catch_ctrl();
    let errors = worker::spawn(rl, &module, args, &params, rl.config().network.proxy.clone(), options);
    rl.signal_register().reset_ctrlc();

    Ok(errors)
}

pub fn run_router(rl: &mut Shell, dry_run: bool, configs: &HashMap<String, NotificationConfig>, workspace: &str, topic: &str, notification: &Notification) -> Result<()> {
    for (name, config) in configs {
        if config.matches(workspace, topic) {
            let module = rl.library().get(&config.script)?.canonical();
            if dry_run {
                term::info(&format!("Executed {} {:?} (dry-run)", module, name));
            } else {
                let options = options::Opt::collect(&config.options);
                match exec(rl, &config.script, options, notification) {
                    Ok(0) => {
                        let msg = format!("Executed {} {:?}", module, name);
                        term::info(&msg);
                    },
                    Ok(errors) => {
                        let msg = format!("Executed {} {:?} ({} errors)", module, name, errors);
                        term::error(&msg);
                    },
                    Err(err) => {
                        term::error(&format!("Fatal {} {:?}: {}", module, name, err));
                    },
                }
            }
        }
    }
    Ok(())
}
