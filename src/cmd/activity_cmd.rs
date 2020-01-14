use crate::errors::*;

use crate::cmd::Cmd;
use crate::shell::Shell;
use crate::models::*;
use chrono::{Utc, NaiveDateTime, NaiveTime, Duration};
use std::convert::TryFrom;
use std::str::FromStr;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug)]
pub struct TimeSpec {
    datetime: NaiveDateTime,
}

impl FromStr for TimeSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let now = Utc::now().naive_utc();
        let today = NaiveDateTime::new(now.date(), NaiveTime::from_hms(0, 0, 0));

        let datetime = match s {
            "today" => today,
            "yesterday" => today - Duration::days(1),
            // x {second,minute,hour,day,week,month,year}s? ago
            s => NaiveDateTime::from_str(s)?,
        };

        Ok(TimeSpec {
            datetime,
        })
    }
}

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(short="t", long="topic")]
    topic: Option<String>,
    #[structopt(long="since")]
    since: Option<TimeSpec>,
    #[structopt(long="until")]
    until: Option<TimeSpec>,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let since = self.since.map(|t| t.datetime);
        let until = self.until.map(|t| t.datetime);

        let events = Activity::query(rl.db(), &ActivityFilter {
            topic: self.topic,
            since,
            until,
        })?;
        for activity in events {
            let activity = JsonActivity::try_from(activity)?;
            let s = serde_json::to_string(&activity)?;
            println!("{}", s);
        }

        Ok(())
    }
}
