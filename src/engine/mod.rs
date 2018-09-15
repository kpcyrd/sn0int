use errors::*;

use std::fs;
use std::path::PathBuf;
// use std::collections::{HashSet, HashMap};
use std::collections::HashMap;
use std::sync::mpsc;
use engine::ctx::Script;
use engine::metadata::Metadata;
use paths;
use term;
use worker::{self, Event};

pub mod ctx;
pub mod metadata;
pub mod structs;


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

    pub fn reload_modules(&mut self) -> Result<()> {
        let modules = worker::spawn_fn("Loading modules", || {
            self.reload_modules_quiet()?;
            Ok(self.list().len())
        }, true)?;
        term::info(&format!("Loaded {} modules", modules));
        Ok(())
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
                                        .map_err(|_| format_err!("Failed to decode filename"))?; // TODO: remove extension

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

    // TODO: this should return an iter
    pub fn list(&self) -> Vec<&Module> {
        let mut modules = Vec::new();

        for (key, values) in &self.modules {
            if !key.contains("/") {
                continue;
            }

            for v in values {
                modules.push(v);
            }
        }

        modules
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    author: String,
    description: String,
    version: String,
    script: Script,
}

impl Module {
    pub fn load(path: &PathBuf, author: &str, name: &str) -> Result<Module> {
        let code = fs::read_to_string(path)
            .context("Failed to read module")?;

        let metadata = Metadata::parse(&code)
            .context("Failed to parse module metadata")?;

        let script = Script::load_unchecked(code)?;

        Ok(Module {
            name: name.to_string(),
            author: author.to_string(),
            description: metadata.description,
            version: metadata.version,
            script,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn canonical(&self) -> String {
        format!("{}/{}", self.author, self.name)
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn run(&self, tx: mpsc::Sender<Event>) -> Result<()> {
        self.script.run(tx)
    }
}
