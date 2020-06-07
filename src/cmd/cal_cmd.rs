use crate::errors::*;

use chrono::Utc;
use crate::cal::DateArg;
use crate::cal::date::{DateContext, DateSpec};
use crate::cal::time::{DateTimeContext, DateTimeSpec};
use crate::cmd::Cmd;
use crate::models::*;
use crate::shell::Shell;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Show additional months for context
    #[structopt(short="C", long)]
    context: Option<u32>,
    /// Do not group by date
    #[structopt(short="T", long)]
    time: bool,
    args: Vec<DateArg>,
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        if self.time {
            let dts = DateTimeSpec::from_args(&self.args, self.context)
                .context("Failed to parse date spec")?;
            let filter = ActivityFilter {
                topic: None,
                since: Some(dts.start().and_hms(0, 0, 0)),
                until: Some(dts.end().and_hms(23, 59, 59)),
                location: false,
            };
            let events = Activity::query(rl.db(), &filter)?;
            let ctx = DateTimeContext::new(&events, Utc::now().naive_utc());
            println!("{}", dts.to_term_string(&ctx));
        } else {
            let ds = DateSpec::from_args(&self.args, self.context)
                .context("Failed to parse date spec")?;
            let filter = ActivityFilter {
                topic: None,
                since: Some(ds.start().and_hms(0, 0, 0)),
                until: Some(ds.end().and_hms(23, 59, 59)),
                location: false,
            };
            let events = Activity::query(rl.db(), &filter)?;
            let ctx = DateContext::new(&events, Utc::today().naive_utc());
            println!("{}", ds.to_term_string(&ctx));
        }
        Ok(())
    }
}
