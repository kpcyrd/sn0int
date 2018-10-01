use structopt::clap::AppSettings;

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
