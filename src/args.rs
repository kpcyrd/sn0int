use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use crate::cmd;
use crate::errors::*;
use crate::options;
use crate::workspaces::Workspace;
use sn0int_common::ModuleID;
use std::io;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Select a different workspace instead of the default
    #[arg(short = 'w', long="workspace", env="SN0INT_WORKSPACE")]
    pub workspace: Option<Workspace>,

    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,
}

impl Args {
    pub fn is_sandbox(&self) -> bool {
        matches!(self.subcommand, Some(SubCommand::Sandbox(_)))
    }
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Run a module directly
    #[command(name="run")]
    Run(Run),
    /// For internal use
    #[command(name="sandbox")]
    Sandbox(Sandbox),
    /// Login to the registry for publishing
    #[command(name="login")]
    Login(Login),
    /// Create a new module
    #[command(name="new")]
    New(New),
    /// Publish a script to the registry
    #[command(name="publish")]
    Publish(Publish),
    /// Install a module from the registry
    #[command(name="install")]
    Install(Install),
    /// Search in the registry
    #[command(name="search")]
    Search(Search),
    /// The sn0int package manager
    #[command(name="pkg")]
    Pkg(cmd::pkg_cmd::Args),
    /// Insert into the database
    #[command(name="add")]
    Add(cmd::add_cmd::Args),
    /// Select from the database
    #[command(name="select")]
    Select(cmd::select_cmd::Args),
    /// Delete from the database
    #[command(name="delete")]
    Delete(cmd::delete_cmd::Args),
    /// Query logged activity
    #[command(name="activity")]
    Activity(cmd::activity_cmd::Args),
    /// Include entities in the scope
    #[command(name="scope")]
    Scope(cmd::scope_cmd::Args),
    /// Exclude entities from scope
    #[command(name="noscope")]
    Noscope(cmd::noscope_cmd::Args),
    /// Manage autoscope rules
    Autoscope(cmd::autoscope_cmd::Args),
    /// Manage autonoscope rules
    Autonoscope(cmd::autonoscope_cmd::Args),
    /// Rescope all entities based on autonoscope rules
    Rescope(cmd::rescope_cmd::Args),
    /// Manage workspaces
    #[command(name="workspace")]
    Workspace(cmd::workspace_cmd::Args),
    /// Calendar
    #[command(name="cal")]
    Cal(cmd::cal_cmd::Args),
    /// Notify
    #[command(name="notify")]
    Notify(cmd::notify_cmd::Args),
    /// Verify blob storage for corrupt and dangling blobs
    #[command(name="fsck")]
    Fsck(cmd::fsck_cmd::Args),
    /// Export a workspace for external processing
    #[command(name="export")]
    Export(cmd::export_cmd::Args),
    /// Show statistics about your current workspace
    #[command(name="stats")]
    Stats(cmd::stats_cmd::Args),
    /// Run a lua repl
    #[command(name="repl")]
    Repl,
    /// Show paths of various file system locations
    #[command(name="paths")]
    Paths,
    /// Generate shell completions
    #[command(name="completions")]
    Completions(Completions),
}

#[derive(Debug, Parser)]
pub struct Run {
    #[command(flatten)]
    pub run: cmd::run_cmd::Args,
    /// Run a module from a path
    #[arg(short = 'f', long="file")]
    pub file: bool,
    /// Expose stdin to modules
    #[arg(long="stdin")]
    pub stdin: bool,
    /// Automatically grant access to a keyring namespace
    #[arg(long="grant")]
    pub grants: Vec<String>,
    /// Automatically grant access to all requested keys
    #[arg(long="grant-full-keyring")]
    pub grant_full_keyring: bool,
    /// Automatically deny access to all requested keys
    #[arg(long="deny-keyring")]
    pub deny_keyring: bool,
    /// Exit on first error and set exit code
    #[arg(short = 'x', long="exit-on-error")]
    pub exit_on_error: bool,
    /// Set an option
    #[arg(short = 'o', long="option")]
    pub options: Vec<options::Opt>,
    /// Narrow down targeted entities
    #[arg(short = 't', long="target")]
    pub target: Option<String>,
    /// Dump the sandbox init message to stdout instead of running a child process
    #[arg(long="dump-sandbox-init-msg")]
    pub dump_sandbox_init_msg: bool,
}

#[derive(Debug, Parser)]
pub struct Sandbox {
    /// This value is only used for process listings
    _label: String,
}

#[derive(Debug, Parser)]
pub struct Login {
}

#[derive(Debug, Parser)]
pub struct New {
    /// Path to the new file
    pub path: String,
}

#[derive(Debug, Parser)]
pub struct Publish {
    /// The scripts to publish
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct Install {
    /// The script to install
    pub module: ModuleID,
    /// Specify the version, defaults to the latest version
    pub version: Option<String>,
    #[arg(short = 'f', long="force")]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct Search {
    /// Only show modules that aren't installed yet
    #[arg(long="new")]
    pub new: bool,
    /// The search query
    pub query: String,
}

/// Generate shell completions
#[derive(Debug, Parser)]
pub struct Completions {
    pub shell: Shell,
}

impl Completions {
    pub fn generate(&self) -> Result<()> {
        clap_complete::generate(
            self.shell,
            &mut Args::command(),
            "sn0int",
            &mut io::stdout(),
        );
        Ok(())
    }
}
