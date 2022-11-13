use crate::errors::*;

use crate::cmd::Cmd;
use crate::shell::Shell;
use crate::models::*;
use chrono::{Utc, NaiveDateTime, NaiveTime, Duration};
use regex::Regex;
use std::convert::TryFrom;
use std::io;
use std::str::FromStr;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug)]
pub struct TimeSpec {
    datetime: NaiveDateTime,
}

impl TimeSpec {
    fn resolve(s: &str, now: NaiveDateTime) -> Result<Self> {
        let today = NaiveDateTime::new(now.date(), NaiveTime::from_hms_opt(0, 0, 0).expect("Invalid hour/min/sec"));

        let datetime = match s {
            "today" => today,
            "yesterday" => today - Duration::days(1),
            s if s.ends_with(" ago") => {
                let re = Regex::new(r"(\d+) ?(s|seconds?|m|min|minutes?|h|hours?|d|days?|w|weeks?|months?|y|years?) ago").unwrap();

                let caps = re.captures(s)
                    .ok_or_else(|| format_err!("Couldn't parse TimeSpec"))?;

                let n = caps.get(1).unwrap().as_str()
                    .parse::<i64>()
                    .context("Failed to parse number in timespec")?;
                let unit = caps.get(2).unwrap();

                let duration = match unit.as_str() {
                    "s" | "second" | "seconds" => Duration::seconds(n),
                    "m" | "min" | "minute" | "minutes" => Duration::minutes(n),
                    "h" | "hour" | "hours" => Duration::hours(n),
                    "d" | "day" | "days" => Duration::days(n),
                    "w" | "week" | "weeks" => Duration::days(n * 7),
                    "month" | "months" => Duration::days(n * 31),
                    "y" | "year" | "years" => Duration::days(n * 365),
                    _ => unreachable!(),
                };
                now - duration
            },
            s => NaiveDateTime::from_str(s)?,
        };

        Ok(TimeSpec {
            datetime,
        })
    }
}

impl FromStr for TimeSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let now = Utc::now().naive_utc();
        Self::resolve(s, now)
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


#[cfg(test)]
mod tests {
    use super::*;

    fn datetime() -> NaiveDateTime {
        let date = chrono::NaiveDate::from_ymd_opt(2020, 3, 14).unwrap();
        let time = chrono::NaiveTime::from_hms_opt(16, 20, 23).unwrap();
        NaiveDateTime::new(date, time)
    }

    #[test]
    fn test_today() {
        let x = TimeSpec::resolve("today", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-14T00:00:00").unwrap());
    }

    #[test]
    fn test_yesterday() {
        let x = TimeSpec::resolve("yesterday", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-13T00:00:00").unwrap());
    }

    #[test]
    fn test_20_min_ago() {
        let x = TimeSpec::resolve("20min ago", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-14T16:00:23").unwrap());
    }

    #[test]
    fn test_3_days_ago() {
        let x = TimeSpec::resolve("3 days ago", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-11T16:20:23").unwrap());
    }

    #[test]
    fn test_1_week_ago() {
        let x = TimeSpec::resolve("1w ago", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-07T16:20:23").unwrap());
    }

    #[test]
    fn test_3_months_ago() {
        let x = TimeSpec::resolve("3 months ago", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2019-12-12T16:20:23").unwrap());
    }

    #[test]
    fn test_1_year_ago() {
        let x = TimeSpec::resolve("1 year ago", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2019-03-15T16:20:23").unwrap());
    }

    #[test]
    fn test_exact_time() {
        let x = TimeSpec::resolve("2020-03-14T16:20:23", datetime()).unwrap();
        assert_eq!(x.datetime, NaiveDateTime::from_str("2020-03-14T16:20:23").unwrap());
    }
}
