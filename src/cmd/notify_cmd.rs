use crate::errors::*;

use crate::cmd::Cmd;
use crate::engine::Module;
// use crate::models::*;
use crate::notify::{self, Notification};
use crate::options::{self, Opt};
use crate::shell::Shell;
use crate::term;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Manually add a notification to the outbox
    Send(SendArgs),
    /// Show the current outbox
    Outbox,
    /// Execute a module directly instead of sending a message
    Exec(ExecArgs),
    /// Try to deliver all messages in our outbox
    Deliver,
}

#[derive(Debug, StructOpt)]
pub struct SendArgs {
    /// Evaluate the routing rules, but do not actually send a notification
    #[structopt(short="n", long)]
    pub dry_run: bool,
    pub topic: String,
    #[structopt(flatten)]
    pub notification: Notification,
}

#[derive(Debug, StructOpt)]
pub struct ExecArgs {
    pub module: String,
    #[structopt(short="o", long="option")]
    pub options: Vec<options::Opt>,
    #[structopt(short="v", long="verbose", parse(from_occurrences))]
    verbose: u64,
    #[structopt(flatten)]
    pub notification: Notification,
}

fn print_summary(module: &Module, sent: usize, errors: usize) {
        let mut out = if sent == 1 {
            String::from("Sent 1 notification")
        } else {
            format!("Sent {} notifications", sent)
        };

        out.push_str(&format!(" with {}", module.canonical()));

        if errors > 0 {
            out.push_str(&format!(" ({} errors)", errors));
        }

        term::info(&out);
}

fn send(args: SendArgs, rl: &mut Shell) -> Result<()> {
    notify::run_router(rl, &mut term::Term, args.dry_run, &args.topic, &args.notification)?;
    Ok(())
}

fn exec(args: ExecArgs, rl: &mut Shell) -> Result<()> {
    let module = rl.library().get(&args.module)?.clone();
    let options = Opt::collect(&args.options);
    let errors = notify::exec(rl, &module, options, args.verbose, &args.notification)?;
    print_summary(&module, 1, errors);
    Ok(())
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        match self.subcommand {
            Subcommand::Send(args) => send(args, rl),
            Subcommand::Outbox => todo!(),
            Subcommand::Exec(args) => exec(args, rl),
            Subcommand::Deliver => todo!(),
        }
    }
}
