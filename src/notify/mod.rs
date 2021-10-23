mod rules;

use crate::errors::*;
use crate::cmd::run_cmd::prepare_keyring;
use crate::cmd::run_cmd::Params;
use crate::engine::Module;
use crate::options;
use crate::shell::Shell;
use crate::term::SpinLogger;
use crate::worker;
use self::rules::Glob;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use sn0int_common::metadata::Source;
use sn0int_std::blobs::Blob;
use sn0int_std::ratelimits::Ratelimiter;
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
    pub topics: Vec<Glob>,
    pub script: String,
    #[serde(default)]
    pub options: Vec<options::Opt>,
}

fn apply_rule<T>(name: &str, filters: &[T], value: &str, cmp: fn(&T, &str) -> Option<bool>) -> bool {
    if !filters.is_empty() {
        debug!("{} filter is active", name);
        for filter in filters {
            match cmp(filter, value) {
                Some(true) => {
                    debug!("{} was allow-listed", name);
                    return true;
                }
                Some(false) => {
                    debug!("{} was excluded", name);
                    return false;
                }
                _ => (),
            }
        }
        debug!("{} didn't match any rules, skipping", name);
        false
    } else {
        true
    }
}

impl NotificationConfig {
    fn matches(&self, name: &str, workspace: &str, topic: &str) -> bool {
        debug!("Testing notification with rules: {:?}", name);
        if !apply_rule("workspace", &self.workspaces, workspace, |filter, value| if filter == value { Some(true) } else { None }) {
            return false;
        }
        if !apply_rule("topic", &self.topics, topic, |filter, value| filter.matches(value)) {
            return false;
        }
        debug!("Notification matches this config");
        true
    }
}

pub fn trigger_notify_event<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, topic: &str, notification: &Notification) -> Result<()> {
    run_router(rl, spinner, ratelimit, false, topic, notification)
}

fn prepare_arg(notification: &Notification) -> Result<(serde_json::Value, Option<String>, Vec<Blob>)> {
    let arg = serde_json::to_value(notification)?;
    Ok((arg, None, vec![]))
}

pub fn exec(rl: &mut Shell, module: &Module, ratelimit: &mut Ratelimiter, options: HashMap<String, String>, verbose: u64, notification: &Notification) -> Result<usize> {
    let module_name = module.canonical();
    debug!("Setting up notification execution with {:?}", module_name);

    if *module.source() != Some(Source::Notifications) {
        bail!("Module doesn't take notifications as source");
    }

    let params = Params {
        module: None,
        threads: 1,
        verbose,
        stdin: false,
        grants: &[],
        grant_full_keyring: false,
        deny_keyring: false,
        exit_on_error: false,
        proxy: None,
        user_agent: None,
    };

    prepare_keyring(rl.keyring_mut(), module, &params)?;
    let args = vec![prepare_arg(notification)?];

    debug!("Executing notification module {:?}", module_name);
    let errors = worker::spawn(rl, module, ratelimit, args, &params, rl.config().network.proxy, None, options);
    debug!("Notification module {:?} exited with {:?} errors", module_name, errors);

    Ok(errors)
}

pub fn run_router<T: SpinLogger>(rl: &mut Shell, spinner: &mut T, ratelimit: &mut Ratelimiter, dry_run: bool, topic: &str, notification: &Notification) -> Result<()> {
    let configs = rl.config().notifications.clone();

    debug!("Running notification router");
    for (name, config) in configs {
        if rl.signal_register().ctrlc_received() {
            debug!("Exiting notification router due to ctrl-c");
            break;
        }

        if config.matches(&name, rl.workspace(), topic) {
            let module = rl.library().get(&config.script)?.clone();
            if dry_run {
                spinner.success(&format!("Executed {} for {:?} (dry-run)", module.canonical(), name));
            } else {
                let options = options::Opt::collect(&config.options);
                match exec(rl, &module, ratelimit, options, 0, notification) {
                    Ok(0) => {
                        let msg = format!("Executed {} for {:?}", module.canonical(), name);
                        spinner.success(&msg);
                    },
                    Ok(errors) => {
                        let msg = format!("Executed {} for {:?} ({} errors)", module.canonical(), name, errors);
                        spinner.error(&msg);
                    },
                    Err(err) => {
                        spinner.error(&format!("Fatal {} for {:?}: {}", module.canonical(), name, err));
                    },
                }
            }
        }
    }
    debug!("Notification router finished");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mkconfig(topics: &[&str]) -> NotificationConfig {
        let topics = topics.iter()
            .map(|s| s.parse::<Glob>())
            .collect::<Result<Vec<_>>>().unwrap();

        NotificationConfig {
            workspaces: Vec::new(),
            topics,
            script: "some/script".to_string(),
            options: Vec::new(),
        }
    }

    #[test]
    fn test_empty_topic() {
        let config = mkconfig(&[]);
        assert!(config.matches("name", "workspace", "topic"));
    }

    #[test]
    fn test_match_topic() {
        let config = mkconfig(&[
            "db:subdomain:example.com:*",
        ]);
        assert!(config.matches("name", "workspace", "db:subdomain:example.com:update"));
    }

    #[test]
    fn test_not_match_topic() {
        let config = mkconfig(&[
            "db:subdomain:example.com:*",
        ]);
        assert!(!config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_exclude_topic() {
        let config = mkconfig(&[
            "!db:subdomain:example.com:*",
        ]);
        assert!(!config.matches("name", "workspace", "db:subdomain:example.com:update"));
    }

    #[test]
    fn test_exclude_other_topic() {
        let config = mkconfig(&[
            "!db:subdomain:example.com:*",
            "db:subdomain:foobar.com:*",
        ]);
        assert!(config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_no_inverse_does_not_imply_match() {
        let config = mkconfig(&[
            "!db:subdomain:example.com:*",
        ]);
        assert!(!config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_everything_except() {
        let config = mkconfig(&[
            "!db:subdomain:example.com:*",
            "*:*:*:*",
        ]);
        assert!(config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_exclude_everything() {
        let config = mkconfig(&[
            "!*:*:*:*",
        ]);
        assert!(!config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_execute_in_order_1() {
        let config = mkconfig(&[
            "!*:*:*:*",
            "*:*:*:*",
        ]);
        assert!(!config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }

    #[test]
    fn test_execute_in_order_2() {
        let config = mkconfig(&[
            "*:*:*:*",
            "!*:*:*:*",
        ]);
        assert!(config.matches("name", "workspace", "db:subdomain:foobar.com:update"));
    }
}
