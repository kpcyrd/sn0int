use crate::errors::*;

use crate::blobs::Blob;
use crate::config::Config;
use crate::geoip::MaxmindReader;
use crate::json::LuaJsonValue;
use crate::keyring::KeyRingEntry;
use serde_json;
use std::fs;
use std::fmt::Debug;
use std::path::PathBuf;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::engine::ctx::Script;
use sn0int_common::ModuleID;
use sn0int_common::metadata::{Metadata, Source};
use chrootable_https::dns::Resolver;
use crate::psl::PslReader;
use crate::paths;
use std::cmp::Ordering;
use std::path::Path;
use crate::term;
use crate::worker::{self, Event};

pub mod ctx;
pub mod isolation;
pub mod structs;


/// Data that is passed to every script
#[derive(Debug)]
pub struct Environment {
    pub verbose: u64,
    pub keyring: Vec<KeyRingEntry>,
    pub dns_config: Resolver,
    pub proxy: Option<SocketAddr>,
    pub options: HashMap<String, String>,
    pub blobs: Vec<Blob>,
    pub psl: PslReader,
    pub geoip: MaxmindReader,
    pub asn: MaxmindReader,
}

#[derive(Debug)]
pub struct Engine<'a> {
    path: PathBuf,
    modules: HashMap<String, Vec<Module>>,
    config: &'a Config
}

impl<'a> Engine<'a> {
    pub fn new(verbose_init: bool, config: &'a Config) -> Result<Engine> {
        let path = paths::module_dir()?;

        let mut engine = Engine {
            path,
            modules: HashMap::new(),
            config,
        };

        if verbose_init {
            engine.reload_modules()?;
        } else {
            engine.reload_modules_quiet()?;
        }

        Ok(engine)
    }

    pub fn reload_modules(&mut self) -> Result<usize> {
        let modules = worker::spawn_fn("Loading modules", || {
            self.reload_modules_quiet()
                .context("Failed to load modules")?;
            Ok(self.list().len())
        }, true)?;
        term::info(&format!("Loaded {} modules", modules));
        Ok(modules)
    }

    pub fn private_modules(path: &Path) -> Result<bool> {
        let metadata = fs::symlink_metadata(&path)?.file_type();
        if metadata.is_symlink() {
            debug!("Folder is a symlink, flagging modules as private");
            return Ok(true);
        }

        if path.join(".git").exists() {
            debug!("Folder is a git repo, flagging modules as private");
            return Ok(true);
        }

        Ok(false)
    }

    pub fn reload_modules_quiet(&mut self) -> Result<()> {
        self.modules = HashMap::new();

        for author in fs::read_dir(&self.path)? {
            let author = author?;
            let path = author.path();

            if !path.is_dir() {
                continue;
            }

            let private_modules = Self::private_modules(&path)?;

            let author_name = author.file_name()
                                    .into_string()
                                    .map_err(|_| format_err!("Failed to decode filename"))?;

            self.load_module_folder(&path, &author_name, private_modules)?;
        }

        for (author, folder) in &self.config.namespaces {
            let folder = if folder.is_absolute() {
                folder.to_owned()
            } else {
                let folder = folder.strip_prefix("~/")
                    .unwrap_or(&folder);

                dirs::home_dir()
                    .ok_or_else(|| format_err!("Failed to find home folder"))?
                    .join(folder)
            };

            self.load_module_folder(&folder, &author, true)?;
        }

        Ok(())
    }

    pub fn load_module_folder(&mut self, folder: &Path, author_name: &str, private_modules: bool) -> Result<()> {
        debug!("Loading modules from {:?}", folder);

        for module in fs::read_dir(folder)? {
            let module = module?;
            let module_name = module.file_name()
                                    .into_string()
                                    .map_err(|_| format_err!("Failed to decode filename"))?;

            // find last instance of .lua in filename, if any
            let (module_name, ext) = if let Some(idx) = module_name.rfind(".lua") {
                module_name.split_at(idx)
            } else {
                // TODO: show warning
                continue;
            };

            // if .lua is not at the end, skip
            if ext != ".lua" {
                // TODO: show warning
                continue;
            }

            if let Err(err) = self.load_single_module(&module.path(), &author_name, &module_name, private_modules) {
                let root = err.find_root_cause();
                term::warn(&format!("Failed to load {}/{}: {}", author_name, module_name, root));
            }
        }

        Ok(())
    }

    pub fn load_single_module(&mut self, path: &Path, author_name: &str, module_name: &str, private_module: bool) -> Result<()> {
        let module_name = module_name.to_string();
        let module = Module::load(path, &author_name, &module_name, private_module)
            .context(format!("Failed to parse {}/{}", author_name, module_name))?;

        for key in &[&module_name, &format!("{}/{}", author_name, module_name)] {
            if !self.modules.contains_key(*key) {
                self.modules.insert(key.to_string(), Vec::new());
            }

            let vec = self.modules.get_mut(*key).unwrap();
            vec.push(module.clone());
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Module> {
        if let Some(module) = self.get_opt(name)? {
            Ok(module)
        } else {
            bail!("Module not found")
        }
    }

    pub fn get_opt(&self, name: &str) -> Result<Option<&Module>> {
        if let Some(modules) = self.modules.get(name) {
            if modules.len() != 1 {
                bail!("Ambiguous name: {:?}", modules)
            } else {
                Ok(Some(&modules[0]))
            }
        } else {
            Ok(None)
        }
    }

    pub fn list(&self) -> Vec<&Module> {
        let mut modules: Vec<_> = self.modules.iter()
            .filter(|(key, _)| key.contains('/'))
            .flat_map(|(_, v)| v.iter())
            .collect();
        modules.sort_by(|a, b| a.cmp_canonical(b));
        modules
    }

    pub fn variants(&self) -> Vec<String> {
        self.modules.iter()
            .filter(|(_, values)| values.len() == 1)
            .map(|(key, _)| key.to_owned())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    name: String,
    author: String,
    description: String,
    version: String,
    source: Option<Source>,
    keyring_access: Vec<String>,
    private_module: bool,
    script: Script,
}

impl Module {
    pub fn load(path: &Path, author: &str, name: &str, private_module: bool) -> Result<Module> {
        debug!("Loading lua module {}/{} from {:?}", author, name, path);
        let code = fs::read_to_string(path)
            .context("Failed to read module")?;

        let metadata = code.parse::<Metadata>()
            .context("Failed to parse module metadata")?;

        let script = Script::load_unchecked(code)?;

        Ok(Module {
            name: name.to_string(),
            author: author.to_string(),
            description: metadata.description,
            version: metadata.version,
            source: metadata.source,
            keyring_access: metadata.keyring_access,
            private_module,
            script,
        })
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn canonical(&self) -> String {
        format!("{}/{}", self.author, self.name)
    }

    #[inline]
    pub fn id(&self) -> ModuleID {
        ModuleID {
            author: self.author.to_string(),
            name: self.name.to_string(),
        }
    }

    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[inline]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[inline]
    pub fn source(&self) -> &Option<Source> {
        &self.source
    }

    #[inline]
    pub fn keyring_access(&self) -> &[String] {
        &self.keyring_access
    }

    #[inline]
    pub fn is_private(&self) -> bool {
        self.private_module
    }

    pub fn run(&self, env: Environment, reporter: Arc<Mutex<Box<Reporter>>>, arg: LuaJsonValue) -> Result<()> {
        debug!("Executing lua script {}", self.canonical());
        self.script.run(env, reporter, arg.into())
    }

    #[inline]
    fn cmp_canonical(&self, other: &Module) -> Ordering {
        if self.author == other.author {
            self.name.cmp(&other.name)
        } else {
            self.author.cmp(&other.author)
        }
    }

    pub fn source_equals(&self, other: &str) -> bool {
        match self.source() {
            Some(source) => source.group_as_str() == other,
            None => other == "",
        }
    }
}

pub trait Reporter: Debug {
    fn send(&mut self, event: &Event) -> Result<()>;

    fn recv(&mut self) -> Result<serde_json::Value>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug)]
    pub struct DummyReporter;

    impl DummyReporter {
        pub fn new() -> Arc<Mutex<Box<Reporter>>> {
            Arc::new(Mutex::new(Box::new(DummyReporter)))
        }
    }

    impl Reporter for DummyReporter {
        fn send(&mut self, _event: &Event) -> Result<()> {
            Ok(())
        }

        fn recv(&mut self) -> Result<serde_json::Value> {
            unimplemented!("DummyReporter::recv doesn't exist")
        }
    }
}
