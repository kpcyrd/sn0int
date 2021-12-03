use crate::errors::*;

use crate::autonoscope::AutoRule;
use crate::cmd::Cmd;
use crate::db::Database;
use crate::filters::Filter;
use crate::shell::Shell;
use std::fmt;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use crate::models::*;
use crate::utils;
use crate::term;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(short, long)]
    interactive: bool,
    #[structopt(short="y", long)]
    auto_confirm: bool,
    #[structopt(short="n", long)]
    dry_run: bool,
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

fn rescope_to_queue<T, F1, F2>(update_queue: &mut Vec<(Entity, bool)>, db: &Database, interactive: bool, matches_rule: F1, wrap: F2) -> Result<()>
    where
        T: Model + Scopable + fmt::Debug,
        F1: Fn(&T) -> Result<Option<bool>>,
        F2: Fn(T) -> Entity,
{
    let any_filter = Filter::any();
    let entities = db.filter::<T>(&any_filter)?;

    for entity in entities {
        let currently_scoped = entity.scoped();
        debug!("rescoping entity: {:?}", entity);

        if let Some(should_be) = matches_rule(&entity)? {
            if currently_scoped != should_be {
                let prefix = if should_be {
                    "\x1b[1m[\x1b[32m+\x1b[0;1m]\x1b[0m"
                } else {
                    "\x1b[1m[\x1b[31m-\x1b[0;1m]\x1b[0m"
                };

                println!("{} updating entity {:?} => {:?}: {:?}", prefix, currently_scoped, should_be, entity);
                if interactive && !utils::yes_else_no("Update this entity?")? {
                    continue;
                }

                update_queue.push((wrap(entity), should_be));
            }
        }
    }

    Ok(())
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let rules = rl.db().autonoscope();
        term::success(&format!("Loaded {} rules", rules.len()));

        let mut update_queue = Vec::new();

        rescope_to_queue::<Domain, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.domains() {
                if rule.matches(entity.value.as_str())? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::Domain)?;
        rescope_to_queue::<Subdomain, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.domains() {
                if rule.matches(entity.value.as_str())? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::Subdomain)?;

        rescope_to_queue::<IpAddr, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::IpAddr)?;

        rescope_to_queue::<Url, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.domains() {
                if rule.matches(entity)? {
                    return Ok(Some(rule.scoped));
                }
            }
            for rule in rules.urls() {
                if rule.matches(entity)? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::Url)?;
        rescope_to_queue::<Port, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::Port)?;
        rescope_to_queue::<Netblock, _, _>(&mut update_queue, rl.db(), self.interactive, |entity| {
            for rule in rules.ips() {
                if rule.matches(entity)? {
                    return Ok(Some(rule.scoped));
                }
            }
            Ok(None)
        }, Entity::Netblock)?;

        if update_queue.is_empty() {
            term::success("Nothing has changed, not updating database");
        } else {
            let confirm = if self.dry_run {
                false
            } else if self.auto_confirm {
                true
            } else {
                utils::no_else_yes(&format!("Apply {} changes to scope now?", update_queue.len()))?
            };

            if confirm {
                term::info(&format!("Applying now: {} changes", update_queue.len()));

                for (update, value) in update_queue {
                    update.set_scoped(rl.db(), value)?;
                }
            } else {
                term::info("Database not updated");
            }
        }

        Ok(())
    }
}
