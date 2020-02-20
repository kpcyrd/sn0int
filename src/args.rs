use structopt::clap::{AppSettings, Shell};
use sn0int_common::ModuleID;
use crate::cmd;
use crate::options;
use crate::workspaces::Workspace;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Select a different workspace instead of the default
    #[structopt(short="w", long="workspace")]
    pub workspace: Option<Workspace>,

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
    /// Run a module directly
    #[structopt(name="run")]
    Run(Run),
    /// For internal use
    #[structopt(name="sandbox")]
    Sandbox(Sandbox),
    /// Login to the registry for publishing
    #[structopt(name="login")]
    Login(Login),
    /// Create a new module
    #[structopt(name="new")]
    New(New),
    /// Publish a script to the registry
    #[structopt(name="publish")]
    Publish(Publish),
    /// Install a module from the registry
    #[structopt(name="install")]
    Install(Install),
    /// Search in the registry
    #[structopt(name="search")]
    Search(Search),
    /// The sn0int package manager
    #[structopt(name="pkg")]
    Pkg(cmd::pkg_cmd::Args),
    /// Insert into the database
    #[structopt(name="add")]
    Add(cmd::add_cmd::Args),
    /// Select from the database
    #[structopt(name="select")]
    Select(cmd::select_cmd::Args),
    /// Delete from the database
    #[structopt(name="delete")]
    Delete(cmd::delete_cmd::Args),
    /// Query logged activity
    #[structopt(name="activity")]
    Activity(cmd::activity_cmd::Args),
    /// Include entities in the scope
    #[structopt(name="scope")]
    Scope(cmd::scope_cmd::Args),
    /// Exclude entities from scope
    #[structopt(name="noscope")]
    Noscope(cmd::noscope_cmd::Args),
    /// Manage workspaces
    #[structopt(name="workspace")]
    Workspace(cmd::workspace_cmd::Args),
    /// Verify blob storage for corrupt and dangling blobs
    #[structopt(name="fsck")]
    Fsck(cmd::fsck_cmd::Args),
    /// Export a workspace for external processing
    #[structopt(name="export")]
    Export(cmd::export_cmd::Args),
    /// Run a lua repl
    #[structopt(name="repl")]
    Repl,
    /// Show paths of various file system locations
    #[structopt(name="paths")]
    Paths,
    /// Generate shell completions
    #[structopt(name="completions")]
    Completions(Completions),
}

#[derive(Debug, StructOpt)]
pub struct Run {
    /// Execute a module that has been installed
    pub module: String,
    /// Run a module from a path
    #[structopt(short="f", long="file")]
    pub file: bool,
    /// Run modules concurrently
    #[structopt(short="j", long="threads", default_value="1")]
    pub threads: usize,
    /// Verbose logging, once to print inserts even if they don't add new
    /// data, twice to activate the debug() function
    #[structopt(short="v", long="verbose", parse(from_occurrences))]
    pub verbose: u64,
    /// Expose stdin to modules
    #[structopt(long="stdin")]
    pub stdin: bool,
    /// Automatically grant access to a keyring namespace
    #[structopt(long="grant")]
    pub grants: Vec<String>,
    /// Automatically grant access to all requested keys
    #[structopt(long="grant-full-keyring")]
    pub grant_full_keyring: bool,
    /// Automatically deny access to all requested keys
    #[structopt(long="deny-keyring")]
    pub deny_keyring: bool,
    /// Exit on first error and set exit code
    #[structopt(short="x", long="exit-on-error")]
    pub exit_on_error: bool,
    /// Set an option
    #[structopt(short="o", long="option")]
    pub options: Vec<options::Opt>,
    /// Narrow down targeted entities
    #[structopt(short="t", long="target")]
    pub target: Option<String>,
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
pub struct New {
    /// Path to the new file
    pub path: String,
}

#[derive(Debug, StructOpt)]
pub struct Publish {
    /// The scripts to publish
    #[structopt(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Install {
    /// The script to install
    pub module: ModuleID,
    /// Specify the version, defaults to the latest version
    pub version: Option<String>,
    #[structopt(short="f", long="force")]
    pub force: bool,
}

#[derive(Debug, StructOpt)]
pub struct Search {
    /// Only show modules that aren't installed yet
    #[structopt(long="new")]
    pub new: bool,
    /// The search query
    pub query: String,
}

#[derive(Debug, StructOpt)]
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}
