use crate::errors::*;

use crate::cmd::Cmd;
use crate::shell::Shell;
use crate::models::*;
use chrono::{Utc, NaiveDateTime, NaiveTime, Duration};
use std::convert::TryFrom;
use std::io;
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
    /// Only query events for a given topic
    #[structopt(short="t", long="topic")]
    topic: Option<String>,
    /// Only query events starting from that datetime
    #[structopt(long="since")]
    since: Option<TimeSpec>,
    /// Only query events until this datetime
    #[structopt(long="until")]
    until: Option<TimeSpec>,
    /// Try to select the previous event before --since as an initial state
    #[structopt(short="i", long="initial")]
    initial: bool,
    /// Only query events that are tied to a location
    #[structopt(short="l", long="location")]
    location: bool,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let filter = ActivityFilter {
            topic: self.topic,
            since: self.since.map(|t| t.datetime),
            until: self.until.map(|t| t.datetime),
            location: self.location,
        };

        let mut stdout = io::stdout();
        let events = Activity::query(rl.db(), &filter)?;

        if self.initial {
            if let Some(first) = events.get(0) {
                let previous = Activity::previous(rl.db(), first, &filter)?;
                if let Some(previous) = previous {
                    let mut previous = JsonActivity::try_from(previous)?;
                    previous.initial = true;
                    previous.write_to(&mut stdout)?;
                }
            }
        }

        for activity in events {
            let activity = JsonActivity::try_from(activity)?;
            activity.write_to(&mut stdout)?;
        }

        Ok(())
    }
}
