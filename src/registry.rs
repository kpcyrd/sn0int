use crate::errors::*;
use crate::args::{Args, Publish, Install, Search};
use crate::api::Client;
use crate::auth;
use crate::config::Config;
use crate::engine::{Engine, Module};
use colored::Colorize;
use separator::Separatable;
use sn0int_common::ModuleID;
use sn0int_common::api::ModuleInfoResponse;
use sn0int_common::metadata::Metadata;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::paths;
use crate::term;
use crate::worker::{self, Task, EventSender, LogEvent};


pub struct Updater {
    client: Client,
}

impl Updater {
    pub fn new(config: &Config) -> Result<Updater> {
        let client = Client::new(&config)?;
        Ok(Updater {
            client,
        })
    }

    #[inline]
    pub fn query_module(&self, module: &ModuleID) -> Result<ModuleInfoResponse> {
        self.client.query_module(module)
    }

    pub fn install(&self, install: &Install) -> Result<String> {
        let version = match install.version {
            Some(ref version) => version.to_string(),
            None => self.client.query_module(&install.module)
                        .context("Failed to query module infos")?
                        .latest
                        .ok_or_else(|| format_err!("Module doesn't have a latest version"))?,
        };

        let module = self.client.download_module(&install.module, &version)
            .context("Failed to download module")?;

        let path = paths::module_dir()?
            .join(format!("{}/{}.lua", install.module.author,
                                       install.module.name));

        fs::create_dir_all(path.parent().unwrap())
            .context("Failed to create folder")?;

        fs::write(&path, module.code)
            .context(format_err!("Failed to write to {:?}", path))?;

        Ok(version)
    }
}

pub fn run_publish(_args: &Args, publish: &Publish, config: &Config) -> Result<()> {
    let session = auth::load_token()
        .context("Failed to load auth token")?;

    let mut client = Client::new(&config)?;
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

        let code = fs::read_to_string(path)
            .context("Failed to read module")?;
        let metadata = code.parse::<Metadata>()?;

        let label = format!("Uploading {} {} ({:?})", name, metadata.version, path);
        match worker::spawn_fn(&label, || {
            client.publish_module(&name, code.to_string())
        }, true) {
            Ok(result) => term::info(&format!("Published {}/{} {} ({:?})",
                                              result.author,
                                              result.name,
                                              result.version,
                                              path)),
            Err(err) => term::error(&format!("Failed to publish {} {} ({:?}): {}",
                                             name,
                                             metadata.version,
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
        let version = self.client.install(&self.install)?;
        let label = format!("installed v{}", version);
        tx.log(LogEvent::Success(label));
        Ok(())
    }
}


pub fn run_install(arg: &Install, config: &Config) -> Result<()> {
    let label = format!("Installing {}", arg.module);
    worker::spawn_fn(&label, || {
        let client = Updater::new(config)?;
        client.install(arg)?;
        Ok(())
    }, false)
}

pub struct UpdateTask {
    module: Module,
    config: Config,
}

impl UpdateTask {
    pub fn new(module: Module, config: Config) -> UpdateTask {
        UpdateTask {
            module,
            config,
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

        let client = Updater::new(&self.config)?;

        let infos = client.query_module(&self.module.id())?;
        debug!("Latest version: {:?}", infos);
        let latest = infos.latest.ok_or_else(|| format_err!("Module doesn't have any released versions"))?;

        if installed != latest {
            let label = format!("Updating {}: v{} -> v{}", self.name(), installed, latest);
            tx.log(LogEvent::Status(label));

            client.install(&Install {
                module: self.module.id(),
                version: None,
            })?;

            let label = format!("updated v{} -> v{}", installed, latest);
            tx.log(LogEvent::Success(label));
        }

        Ok(())
    }
}

pub fn run_search(engine: &Engine, search: &Search, config: &Config) -> Result<()> {
    let client = Client::new(&config)?;

    let label = format!("Searching {:?}", search.query);
    let modules = worker::spawn_fn(&label, || {
        client.search(&search.query)
    }, true)?;

    for module in &modules {
        let canonical = module.canonical();
        let installed = engine.get_opt(&canonical)?;

        if search.new && installed.is_some() {
            continue;
        }

        println!("{} ({}) - {} downloads{}{}", canonical.green(),
                            module.latest.yellow(),
                            module.downloads.separated_string(),
                            (if module.featured { " [featured]" } else { "" }).cyan(),
                            (if installed.is_some() { " [installed]" } else { "" }).green());
        println!("\t{}", module.description);
    }

    Ok(())
}
