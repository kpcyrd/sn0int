use errors::*;

use geoip::{GeoIP, AsnDB};
use json::LuaJsonValue;
use serde_json;
use std::fs;
use std::fmt::Debug;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use engine::ctx::Script;
use sn0int_common::ModuleID;
use sn0int_common::metadata::{Metadata, Source};
use chrootable_https::dns::DnsConfig;
use psl::Psl;
use paths;
use std::cmp::Ordering;
use term;
use worker::{self, Event};

pub mod ctx;
pub mod isolation;
pub mod structs;


/// Data that is passed to every script
#[derive(Debug)]
pub struct Environment {
  pub dns_config: DnsConfig,
  pub psl: Psl,
  pub geoip: GeoIP,
  pub asn: AsnDB,
}

#[derive(Debug)]
pub struct Engine {
    path: PathBuf,
    modules: HashMap<String, Vec<Module>>,
}

impl Engine {
    pub fn new() -> Result<Engine> {
        let path = paths::module_dir()?;

        let mut engine = Engine {
            path,
            modules: HashMap::new(),
        };

        engine.reload_modules()?;

        Ok(engine)
    }

    pub fn reload_modules(&mut self) -> Result<usize> {
        let modules = worker::spawn_fn("Loading modules", || {
            self.reload_modules_quiet()?;
            Ok(self.list().len())
        }, true)?;
        term::info(&format!("Loaded {} modules", modules));
        Ok(modules)
    }

    pub fn reload_modules_quiet(&mut self) -> Result<()> {
        self.modules = HashMap::new();

        for author in fs::read_dir(&self.path)? {
            let author = author?;
            let author_name = author.file_name()
                                    .into_string()
                                    .map_err(|_| format_err!("Failed to decode filename"))?;
            for module in fs::read_dir(&author.path())? {
                let module = module?;
                let mut module_name = module.file_name()
                                        .into_string()
                                        .map_err(|_| format_err!("Failed to decode filename"))?;

                if module_name.ends_with(".lua") {
                    module_name = module_name[..(module_name.len() - 4)].to_string();
                } else {
                    // TODO: show warning
                    continue;
                }

                let path = module.path();
                let module = Module::load(&path, &author_name, &module_name)
                    .context(format!("Failed to parse {}/{}", author_name, module_name))?;

                for key in &[module_name.clone(), format!("{}/{}", author_name, module_name)] {
                    if !self.modules.contains_key(key) {
                        self.modules.insert(key.to_string(), Vec::new());
                    }

                    let vec = self.modules.get_mut(key).unwrap();
                    vec.push(module.clone());
                }
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Module> {
        if let Some(modules) = self.modules.get(name) {
            if modules.len() != 1 {
                bail!("Ambiguous name: {:?}", modules)
            } else {
                Ok(&modules[0])
            }
        } else {
            bail!("Module not found")
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
    script: Script,
}

impl Module {
    pub fn load(path: &PathBuf, author: &str, name: &str) -> Result<Module> {
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
            script,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn canonical(&self) -> String {
        format!("{}/{}", self.author, self.name)
    }

    pub fn id(&self) -> ModuleID {
        ModuleID {
            author: self.author.to_string(),
            name: self.name.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn source(&self) -> &Option<Source> {
        &self.source
    }

    pub fn run(&self, env: Environment, reporter: Arc<Mutex<Box<Reporter>>>, arg: LuaJsonValue) -> Result<()> {
        debug!("Executing lua script {}", self.canonical());
        self.script.run(env, reporter, arg.into())
    }

    fn cmp_canonical(&self, other: &Module) -> Ordering {
        if self.author == other.author {
            self.name.cmp(&other.name)
        } else {
            self.author.cmp(&other.author)
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
