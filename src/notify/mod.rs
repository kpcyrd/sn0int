use crate::cmd::run_cmd::prepare_keyring;
use crate::cmd::run_cmd::Params;
use crate::engine::Module;
use crate::errors::*;
use crate::options;
use crate::shell::Shell;
use crate::term::SpinLogger;
use crate::worker;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use sn0int_common::metadata::Source;
use sn0int_std::blobs::Blob;
use std::collections::HashMap;
use std::result;
use std::str::FromStr;

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

#[derive(Debug, Clone)]
pub struct Glob {
    patterns: Vec<glob::Pattern>,
    src: String,
}

impl Glob {
    fn matches(&self, topic: &str) -> bool {
        let mut filter = self.patterns.iter();
        let mut topic = topic.split(':');

        loop {
            match (filter.next(), topic.next()) {
                (Some(filter), Some(topic)) => {
                    if !filter.matches(&topic) {
                        return false;
                    }
                }
                (None, None) => return true,
                (_, _) => return false,
            }
        }
    }
}

impl FromStr for Glob {
    type Err = Error;

    fn from_str(s: &str) -> Result<Glob> {
        let patterns = s
            .split(':')
            .map(|s| glob::Pattern::new(s).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        Ok(Glob {
            patterns,
            src: s.to_string(),
        })
    }
}

impl Serialize for Glob {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.src)
    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

fn apply_rule<T>(name: &str, filter: &[T], value: &str, cmp: fn(&T, &str) -> bool) -> bool {
    if !filter.is_empty() {
        debug!("{} filter is active", name);
        if !filter.iter().any(|filter| cmp(filter, value)) {
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
        if !apply_rule("workspace", &self.workspaces, workspace, |filter, value| {
            filter == value
        }) {
            return false;
        }
        if !apply_rule("topic", &self.topics, topic, |filter, value| {
            filter.matches(value)
        }) {
            return false;
        }
        debug!("notification matches this config");
        true
    }
}

pub fn trigger_notify_event<T: SpinLogger>(
    rl: &mut Shell,
    spinner: &mut T,
    topic: &str,
    notification: &Notification,
) -> Result<()> {
    run_router(rl, spinner, false, topic, notification)
}

fn prepare_arg(
    notification: &Notification,
) -> Result<(serde_json::Value, Option<String>, Vec<Blob>)> {
    let arg = serde_json::to_value(notification)?;
    Ok((arg, None, vec![]))
}

pub fn exec(
    rl: &mut Shell,
    module: &Module,
    options: HashMap<String, String>,
    verbose: u64,
    notification: &Notification,
) -> Result<usize> {
    if *module.source() != Some(Source::Notifications) {
        bail!("Module doesn't take notifications as source");
    }

    let params = Params {
        threads: 1,
        verbose,
        stdin: false,
        grants: &[],
        grant_full_keyring: false,
        deny_keyring: false,
        exit_on_error: false,
    };

    prepare_keyring(rl.keyring_mut(), &module, &params)?;
    let args = vec![prepare_arg(&notification)?];

    rl.signal_register().catch_ctrl();
    let errors = worker::spawn(
        rl,
        &module,
        args,
        &params,
        rl.config().network.proxy.clone(),
        options,
    );
    rl.signal_register().reset_ctrlc();

    Ok(errors)
}

pub fn run_router<T: SpinLogger>(
    rl: &mut Shell,
    spinner: &mut T,
    dry_run: bool,
    topic: &str,
    notification: &Notification,
) -> Result<()> {
    let configs = rl.config().notifications.clone();

    for (name, config) in configs {
        if config.matches(rl.workspace(), topic) {
            let module = rl.library().get(&config.script)?.clone();
            if dry_run {
                spinner.success(&format!(
                    "Executed {} {:?} (dry-run)",
                    module.canonical(),
                    name
                ));
            } else {
                let options = options::Opt::collect(&config.options);
                match exec(rl, &module, options, 0, notification) {
                    Ok(0) => {
                        let msg = format!("Executed {} {:?}", module.canonical(), name);
                        spinner.success(&msg);
                    }
                    Ok(errors) => {
                        let msg = format!(
                            "Executed {} {:?} ({} errors)",
                            module.canonical(),
                            name,
                            errors
                        );
                        spinner.error(&msg);
                    }
                    Err(err) => {
                        spinner.error(&format!("Fatal {} {:?}: {}", module.canonical(), name, err));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn match_topic_str(filter: &str, value: &str) -> bool {
        let filter: Glob = filter.parse().unwrap();
        filter.matches(value)
    }

    #[test]
    fn test_match_topic_exact() {
        assert!(match_topic_str("topic:hello-world", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_starts_with() {
        assert!(match_topic_str("topic:*", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_ends_with() {
        assert!(match_topic_str("*:hello-world", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_one_wildcard_one_section() {
        assert!(match_topic_str("a:*:z", "a:b:z"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections() {
        assert!(!match_topic_str("a:*:z", "a:b:c:z"));
    }

    #[test]
    fn test_match_topic_two_wildcards_two_sections() {
        assert!(match_topic_str("a:*:*:z", "a:b:c:z"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections_start() {
        assert!(!match_topic_str("a:*", "a:b:c"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections_end() {
        assert!(!match_topic_str("*:z", "b:c:z"));
    }

    #[test]
    fn test_match_topic_many_wildcards() {
        assert!(match_topic_str("a:*:*:d:e:*:g:*:z", "a:b:c:d:e:f:g:h:z"));
    }

    #[test]
    fn test_match_topic_empty_filter() {
        assert!(!match_topic_str("", "abc"));
    }
}
