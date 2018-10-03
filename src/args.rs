use structopt::clap::AppSettings;
use sn0int_common::ModuleID;


#[derive(Debug, StructOpt)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short="w", long="workspace")]
    /// Select a different workspace instead of the default
    pub workspace: Option<String>,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

impl Args {
    pub fn is_sandbox(&self) -> bool {
        match self.subcommand {
            Some(SubCommand::Sandbox(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    #[structopt(author="", name="run")]
    /// Run a module directly
    Run(Run),
    #[structopt(author="", name="sandbox")]
    /// For internal use
    Sandbox(Sandbox),
    #[structopt(author="", name="login")]
    /// Login to the registry for publishing
    Login(Login),
    #[structopt(author="", name="publish")]
    /// Publish a script to the registry
    Publish(Publish),
    #[structopt(author="", name="install")]
    /// Install a module from the registry
    Install(Install),
    #[structopt(author="", name="search")]
    /// Search in the registry
    Search(Search),
}

#[derive(Debug, StructOpt)]
pub struct Run {
    pub module: Option<String>,
    #[structopt(short="f", long="file", conflicts_with="module")]
    pub file: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct Sandbox {
    /// This value is only used for process listings
    label: String,
}

#[derive(Debug, StructOpt)]
pub struct Login {
}

#[derive(Debug, StructOpt)]
pub struct Publish {
    /// The script to publish
    pub path: String,
}

#[derive(Debug, StructOpt)]
pub struct Install {
    /// The script to install
    pub module: ModuleID,
    /// Specify the version, defaults to the latest version
    pub version: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct Search {
    /// The search query
    pub query: String,
}
