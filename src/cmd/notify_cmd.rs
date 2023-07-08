use clap::{ArgAction, Parser};
use crate::cmd::Cmd;
use crate::engine::Module;
use crate::errors::*;
use crate::notify::{self, Notification};
use crate::options::{self, Opt};
use crate::shell::Shell;
use crate::term;
use sn0int_std::ratelimits::Ratelimiter;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Parser)]
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

#[derive(Debug, Parser)]
pub struct SendArgs {
    /// Evaluate the routing rules, but do not actually send a notification
    #[arg(short = 'n', long)]
    pub dry_run: bool,
    pub topic: String,
    #[command(flatten)]
    pub notification: Notification,
}

#[derive(Debug, Parser)]
pub struct ExecArgs {
    pub module: String,
    #[arg(short = 'o', long="option")]
    pub options: Vec<options::Opt>,
    #[arg(short = 'v', long="verbose", action(ArgAction::Count))]
    verbose: u8,
    #[command(flatten)]
    pub notification: Notification,
}

fn print_summary(module: &Module, sent: usize, errors: usize) {
    let mut out = if sent == 1 {
        String::from("Sent 1 notification")
    } else {
        format!("Sent {} notifications", sent)
    };

    write!(out, " with {}", module.canonical()).expect("out of memory");

    if errors > 0 {
        write!(out, " ({} errors)", errors).expect("out of memory");
    }

    term::info(&out);
}

fn send(args: SendArgs, rl: &mut Shell) -> Result<()> {
    rl.signal_register().catch_ctrl();
    notify::run_router(rl, &mut term::Term, &mut Ratelimiter::new(), args.dry_run, &args.topic, &args.notification)?;
    rl.signal_register().reset_ctrlc();
    Ok(())
}

fn exec(args: ExecArgs, rl: &mut Shell) -> Result<()> {
    let module = rl.library().get(&args.module)?.clone();
    let options = Opt::collect(&args.options);

    rl.signal_register().catch_ctrl();
    let errors = notify::exec(rl, &module, &mut Ratelimiter::new(), options, args.verbose, &args.notification)?;
    rl.signal_register().reset_ctrlc();

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
