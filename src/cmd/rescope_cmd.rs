use crate::errors::*;

use crate::autonoscope::{AutoRule, ToRule};
use crate::cmd::Cmd;
use crate::db::Database;
use crate::filters::{Filter, Target};
use crate::shell::Shell;
use std::collections::HashSet;
use std::fmt;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;
use crate::utils;
use crate::term;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Run rules interactively
    #[structopt(short, long)]
    interactive: bool,
    /// Automatically apply changes to database
    #[structopt(short="y", long)]
    auto_confirm: bool,
    /// Only show changes, do not apply them to the database
    #[structopt(short="n", long)]
    dry_run: bool,
    /// Only rescope entities matching specific filter
    #[structopt(subcommand)]
    target: Option<Target>,
}

enum Entity {
    Domain(Domain),
    Subdomain(Subdomain),
    IpAddr(IpAddr),
    Url(Url),
    Port(Port),
    Netblock(Netblock),
}

impl Entity {
    fn set_scoped(&self, db: &Database, value: bool) -> Result<()> {
        match self {
            Entity::Domain(entity) => entity.set_scoped(db, value),
            Entity::Subdomain(entity) => entity.set_scoped(db, value),
            Entity::IpAddr(entity) => entity.set_scoped(db, value),
            Entity::Url(entity) => entity.set_scoped(db, value),
            Entity::Port(entity) => entity.set_scoped(db, value),
            Entity::Netblock(entity) => entity.set_scoped(db, value),
        }
    }
}

enum Input {
    Yes,
    No,
    Done,
    Always,
    Never,
}

fn get_input() -> Result<Input> {
    loop {
        let input = utils::question_opt("Update this entity? [Y/n/d/a/x/?]")?;
        let input = input.map(|s| s.to_lowercase());
        match input.as_deref() {
            Some("y") | None => return Ok(Input::Yes),
            Some("n") => return Ok(Input::No),
            Some("d") => return Ok(Input::Done),
            Some("a") => return Ok(Input::Always),
            Some("x") => return Ok(Input::Never),
            Some("?") => {
                term::success("y -> yes, apply this change");
                term::success("n -> no, skip this change");
                term::success("d -> done, skip this and further changes");
                term::success("a -> always, apply every change caused by this specific rule");
                term::success("x -> never, skip every change caused by this specific rule");
            },
            Some(input) => term::error(&format!("Unrecognized input: {:?}", input)),
        }
    }
}

#[derive(Default)]
struct Context {
    update_queue: Vec<(Entity, bool)>,
    always_rules: HashSet<(&'static str, String)>,
    never_rules: HashSet<(&'static str, String)>,
    done: bool,
    target: Option<Target>,
}

fn rescope_to_queue<T, F1, F2, F3>(ctx: &mut Context, db: &Database, interactive: bool, get_filter: F1, matches_rule: F2, wrap: F3) -> Result<()>
    where
        T: Model + Scopable + fmt::Debug,
        F1: Fn(&Target) -> Option<&Filter>,
        F2: Fn(&T) -> Result<Option<((&'static str, String), bool)>>,
        F3: Fn(T) -> Entity,
{
    // do nothing if we're already done
    if ctx.done {
        return Ok(());
    }

    // check if there are filters to be applied
    let filter = if let Some(target) = &ctx.target {
        if let Some(filter) = get_filter(target) {
            // we've selected this specific entity type and there's a filter
            filter.parse_optional()
                .context("Filter is invalid")?
        } else {
            // we do not wish to process this entity type
            return Ok(());
        }
    } else {
        // any entity type is fine
        Filter::any()
    };

    let entities = db.filter::<T>(&filter)?;

    for entity in entities {
        let currently_scoped = entity.scoped();
        debug!("rescoping entity: {:?}", entity);

        if let Some((rule, should_be)) = matches_rule(&entity)? {
            // check if we're actively ignoring this rule
            if ctx.never_rules.contains(&rule) {
                continue;
            }

            if currently_scoped != should_be {
                let prefix = if should_be {
                    "\x1b[1m[\x1b[32m+\x1b[0;1m]\x1b[0m"
                } else {
                    "\x1b[1m[\x1b[31m-\x1b[0;1m]\x1b[0m"
                };

                println!("{} Setting entity {:?} => {:?}: {:?}", prefix, currently_scoped, should_be, entity);

                // check if we're auto-accepting this rule
                let input = if ctx.always_rules.contains(&rule) {
                    Input::Yes
                } else if interactive {
                    get_input()?
                } else {
                    Input::Yes
                };

                // process user input
                let input = match input {
                    Input::Always => {
                        ctx.always_rules.insert(rule);
                        Input::Yes
                    },
                    Input::Never => {
                        ctx.never_rules.insert(rule);
                        Input::No
                    },
                    Input::Done => {
                        ctx.done = true;
                        break;
                    },
                    input => input,
                };

                if let Input::Yes = input {
                    ctx.update_queue.push((wrap(entity), should_be));
                }
            }
        }
    }

    Ok(())
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let rules = rl.db().autonoscope();
        term::success(&format!("Loaded {} rules", rules.len()));

        let mut ctx = Context {
            target: self.target,
            ..Default::default()
        };

        rescope_to_queue::<Domain, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.domains(), |entity| {
            for rule in rules.domains() {
                if rule.matches(entity.value.as_str())? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::Domain)?;
        rescope_to_queue::<Subdomain, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.subdomains(), |entity| {
            for rule in rules.domains() {
                if rule.matches(entity.value.as_str())? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::Subdomain)?;

        rescope_to_queue::<IpAddr, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.ipaddrs(), |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::IpAddr)?;

        rescope_to_queue::<Url, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.urls(), |entity| {
            for rule in rules.domains() {
                if rule.matches(entity)? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            for rule in rules.urls() {
                if rule.matches(entity)? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::Url)?;
        rescope_to_queue::<Port, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.ports(), |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::Port)?;
        rescope_to_queue::<Netblock, _, _, _>(&mut ctx, rl.db(), self.interactive, |t| t.netblocks(), |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some((rule.to_rule(), rule.scoped)));
                }
            }
            Ok(None)
        }, Entity::Netblock)?;

        if ctx.update_queue.is_empty() {
            term::success("Nothing has changed, not updating database");
        } else {
            let confirm = if self.dry_run {
                false
            } else if self.auto_confirm {
                true
            } else {
                utils::no_else_yes(&format!("Apply {} changes to scope now?", ctx.update_queue.len()))?
            };

            if confirm {
                term::info(&format!("Applying {} changes to database", ctx.update_queue.len()));

                for (update, value) in ctx.update_queue {
                    update.set_scoped(rl.db(), value)?;
                }
            } else {
                term::info("Database not updated");
            }
        }

        Ok(())
    }
}
