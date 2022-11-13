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
    /// Group events in 12 min slices
    #[structopt(short="T", long, group = "view")]
    time: bool,
    /// Group events by hour
    #[structopt(short="H", long, group = "view")]
    hourly: bool,
    args: Vec<DateArg>,
}

impl Cmd for Args {
    #[inline]
    fn run(self, rl: &mut Shell) -> Result<()> {
        if self.time || self.hourly {
            let dts = DateTimeSpec::from_args(&self.args, self.context)
                .context("Failed to parse date spec")?;
            let filter = ActivityFilter {
                topic: None,
                since: Some(dts.start().and_hms_opt(0, 0, 0).expect("Invalid hour/min/sec")),
                until: Some(dts.end().and_hms_opt(23, 59, 59).expect("Invalid hour/min/sec")),
                location: false,
            };
            let events = Activity::query(rl.db(), &filter)?;

            let (slice_width, slice_duration) = if self.hourly {
                (3, 60)
            } else {
                (1, 12)
            };

            let ctx = DateTimeContext::new(&events, Utc::now().naive_utc(), slice_width, slice_duration);
            println!("{}", dts.to_term_string(&ctx));
        } else {
            let ds = DateSpec::from_args(&self.args, self.context)
                .context("Failed to parse date spec")?;
            let filter = ActivityFilter {
                topic: None,
                since: Some(ds.start().and_hms_opt(0, 0, 0).expect("Invalid hour/min/sec")),
                until: Some(ds.end().and_hms_opt(23, 59, 59).expect("Invalid hour/min/sec")),
                location: false,
            };
            let events = Activity::query(rl.db(), &filter)?;
            let ctx = DateContext::new(&events, Utc::now().date_naive());
            println!("{}", ds.to_term_string(&ctx));
        }
        Ok(())
    }
}
