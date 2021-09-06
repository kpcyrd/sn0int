use crate::errors::*;
use crate::args::{Args, Publish, Install, Search};
use crate::api::Client;
use crate::auth;
use crate::config::Config;
use crate::engine::{Library, Module};
use colored::{Color, Colorize};
use separator::Separatable;
use std::fmt::Write;
use sn0int_common::ModuleID;
use sn0int_common::api::ModuleInfoResponse;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::paths;
use crate::term;
use crate::worker::{self, Task, EventSender, LogEvent};


pub struct Updater {
    client: Client,
}

impl Updater {
    pub fn new(config: &Config) -> Result<Updater> {
        let client = Client::new(config)?;
        Ok(Updater {
            client,
        })
    }

    #[inline]
    pub fn query_module(&self, module: &ModuleID) -> Result<ModuleInfoResponse> {
        self.client.query_module(module)
    }

    fn path(&self, module: &ModuleID) -> Result<PathBuf> {
        let path = paths::module_dir()?
            .join(format!("{}/{}.lua", module.author,
                                       module.name));
        Ok(path)
    }

    pub fn install(&self, install: Install) -> Result<String> {
        if let Some(version) = install.version {
            let module = self.client.download_module(&install.module, &version)
                .context("Failed to download module")?;

            let path = self.path(&install.module)?;

            fs::create_dir_all(path.parent().unwrap())
                .context("Failed to create folder")?;

            fs::write(&path, module.code)
                .context(format_err!("Failed to write to {:?}", path))?;

            Ok(version)
        } else {
            let infos = self.query_module(&install.module)
                        .context("Failed to query module infos")?;

            if !install.force {
                if let Some(redirect) = infos.redirect {
                    return self.install(Install {
                        module: redirect,
                        version: None,
                        force: install.force,
                    });
                }
            }

            let latest = infos
                        .latest
                        .ok_or_else(|| format_err!("Module doesn't have a latest version"))?;
            self.install(Install {
                module: install.module,
                version: Some(latest),
                force: install.force,
            })
        }
    }

    pub fn uninstall(&self, module: &ModuleID) -> Result<()> {
        let path = self.path(module)?;
        fs::remove_file(&path)?;

        // try to delete parent folder if empty
        if let Some(parent) = path.parent() {
            fs::remove_dir(parent).ok();
        }

        Ok(())
    }
}

pub fn run_publish(_args: &Args, publish: &Publish, config: &Config) -> Result<()> {
    let session = auth::load_token()
        .context("Failed to load auth token, login first")?;

    let mut client = Client::new(config)?;
    client.authenticate(session);

    for path in &publish.paths {
        let path = Path::new(path);
        let name = path.file_stem().ok_or_else(|| format_err!("Couldn't get file name"))?;
        let ext = path.extension().ok_or_else(|| format_err!("Couldn't get file extension"))?;

        if ext != "lua" {
            bail!("File extension has to be .lua");
        }

        let name = name.to_os_string().into_string()
            .map_err(|_| format_err!("Failed to decode file name"))?;

        let module = Module::load(path, "anonymous", &name, false)?;

        let label = format!("Uploading {} {} ({:?})", name, module.version(), path);
        match worker::spawn_fn(&label, || {
            client.publish_module(&name, module.code().to_string())
        }, true) {
            Ok(result) => term::info(&format!("Published {}/{} {} ({:?})",
                                              result.author,
                                              result.name,
                                              result.version,
                                              path)),
            Err(err) => term::error(&format!("Failed to publish {} {} ({:?}): {}",
                                             name,
                                             module.version(),
                                             path,
                                             err)),
        }
    }

    Ok(())
}

pub struct InstallTask {
    install: Install,
    client: Arc<Updater>,
}

impl InstallTask {
    pub fn new(install: Install, client: Arc<Updater>) -> InstallTask {
        InstallTask {
            install,
            client,
        }
    }
}

impl Task for InstallTask {
    #[inline]
    fn initial_label(name: &str) -> String {
        format!("Installing {}", name)
    }

    #[inline]
    fn name(&self) -> String {
        self.install.module.to_string()
    }

    fn run(self, tx: &EventSender) -> Result<()> {
        let version = self.client.install(self.install)?;
        let label = format!("installed v{}", version);
        tx.log(LogEvent::Success(label));
        Ok(())
    }
}


pub fn run_install(arg: Install, config: &Config) -> Result<()> {
    let label = format!("Installing {}", arg.module);
    worker::spawn_fn(&label, || {
        let client = Updater::new(config)?;
        client.install(arg)?;
        Ok(())
    }, false)
}

pub struct UpdateTask {
    module: Module,
    client: Arc<Updater>,
}

impl UpdateTask {
    pub fn new(module: Module, client: Arc<Updater>) -> UpdateTask {
        UpdateTask {
            module,
            client,
        }
    }
}

impl Task for UpdateTask {
    #[inline]
    fn initial_label(name: &str) -> String {
        format!("Searching for updates: {}", name)
    }

    #[inline]
    fn name(&self) -> String {
        self.module.canonical()
    }

    fn run(self, tx: &EventSender) -> Result<()> {
        let installed = self.module.version();

        let infos = self.client.query_module(&self.module.id())?;
        debug!("Latest version: {:?}", infos);
        let latest = infos.latest.ok_or_else(|| format_err!("Module doesn't have any released versions"))?;

        if let Some(redirect) = infos.redirect {
            let label = format!("Replacing {}: {}", self.name(), redirect);
            tx.log(LogEvent::Status(label));

            self.client.install(Install {
                module: self.module.id(),
                version: None,
                force: false,
            })?;
            self.client.uninstall(&self.module.id())?;

            let label = format!("replaced with {}", redirect);
            tx.log(LogEvent::Success(label));
        } else if installed != latest {
            let label = format!("Updating {}: v{} -> v{}", self.name(), installed, latest);
            tx.log(LogEvent::Status(label));

            self.client.install(Install {
                module: self.module.id(),
                version: Some(latest.clone()),
                force: false,
            })?;

            let label = format!("updated v{} -> v{}", installed, latest);
            tx.log(LogEvent::Success(label));
        }

        Ok(())
    }
}

#[inline]
fn write_tag(out: &mut String, color: Color, txt: &str) -> Result<()> {
    write!(out, " [{}]", txt.color(color))?;
    Ok(())
}

pub fn run_search(library: &Library, search: &Search, config: &Config) -> Result<()> {
    let client = Client::new(config)?;

    let label = format!("Searching {:?}", search.query);
    let modules = worker::spawn_fn(&label, || {
        client.search(&search.query)
    }, true)?;

    for module in &modules {
        let canonical = module.canonical();
        let installed = library.get_opt(&canonical)?;

        if search.new && installed.is_some() {
            continue;
        }

        let mut out = format!("{}/{} {} - {} downloads",
            module.author.purple(),
            module.name,
            module.latest.blue(),
            module.downloads.separated_string(),
        );

        if module.featured {
            write_tag(&mut out, Color::Cyan, "featured")?;
        }

        if installed.is_some() {
            write_tag(&mut out, Color::Green, "installed")?;
        }

        println!("{}", out.bold());
        println!("    {}", module.description);
    }

    Ok(())
}
