use crate::errors::*;

use chrono::Utc;
use chrono::prelude::*;
use crate::cmd::Cmd;
// use crate::config::Config;
// use crate::db::Database;
use crate::shell::Shell;
// use crate::term;
// use crate::utils;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Show additional months for context
    #[structopt(short="C", long)]
    context: Option<u32>,
    args: Vec<DateArg>,
}

fn days_in_month(year: i32, month: u32) -> i64 {
    let start = Utc.ymd(year, month, 1);
    let end = if month == 12 {
        Utc.ymd(year + 1, 1, 1)
    } else {
        Utc.ymd(year, month + 1, 1)
    };
    end.signed_duration_since(start).num_days()
}

#[derive(Debug)]
enum DateArg {
    Month(u32),
    Num(i32),
}

impl FromStr for DateArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<DateArg> {
        let ds = match s.to_lowercase().as_str() {
            "jan" | "january"   => DateArg::Month(1),
            "feb" | "february"  => DateArg::Month(2),
            "mar" | "march"     => DateArg::Month(3),
            "apr" | "april"     => DateArg::Month(4),
            "may"               => DateArg::Month(5),
            "jun" | "june"      => DateArg::Month(6),
            "jul" | "july"      => DateArg::Month(7),
            "aug" | "august"    => DateArg::Month(8),
            "sep" | "september" => DateArg::Month(9),
            "oct" | "october"   => DateArg::Month(10),
            "nov" | "november"  => DateArg::Month(11),
            "dec" | "december"  => DateArg::Month(12),
            _ => {
                let num = s.parse::<i32>()
                    .context("Input is not a month and not a number")?;
                DateArg::Num(num)
            },
        };
        Ok(ds)
    }
}

enum DateSpec {
    Year(i32),
    YearMonth((i32, u32)),
    YearMonthContext((i32, u32, u32)),
}

impl DateSpec {
    fn from_args(args: &[DateArg], context: Option<u32>) -> Result<DateSpec> {
        if args.len() > 2 {
            bail!("Too many datespec args");
        }

        let today = Utc::today();
        let ds = match (args.get(0), args.get(1), context) {
            (None, _, None) => DateSpec::YearMonth((today.year(), today.month())),
            (None, _, Some(context)) => DateSpec::YearMonthContext((today.year(), today.month(), context)),

            (Some(DateArg::Month(month)), None, None) => DateSpec::YearMonth((today.year(), *month)),
            (Some(DateArg::Num(year)), None, None) => DateSpec::Year(*year),
            (Some(DateArg::Month(month)), Some(DateArg::Num(year)), None) => DateSpec::YearMonth((*year, *month)),
            (Some(DateArg::Num(year)), Some(DateArg::Month(month)), None) => DateSpec::YearMonth((*year, *month)),

            (Some(DateArg::Month(month)), None, Some(context)) => DateSpec::YearMonthContext((today.year(), *month, context)),
            (Some(DateArg::Month(month)), Some(DateArg::Num(year)), Some(context)) => DateSpec::YearMonthContext((*year, *month, context)),
            (Some(DateArg::Num(year)), Some(DateArg::Month(month)), Some(context)) => DateSpec::YearMonthContext((*year, *month, context)),
            _ => bail!("Combination of datespec args is invalid"),
        };
        Ok(ds)
    }
}

const MONTH_LINES: i32 = 7;

fn merge_months(ctx: &Context, months: &[DateSpec]) -> String {
    let mut months = months.iter()
        .map(|ds| {
            let month = ds.to_term_string(ctx);
            month.lines()
                .map(String::from)
                .collect::<VecDeque<_>>()
        })
        .collect::<Vec<_>>();

    let mut out = String::new();
    for i in 0..=MONTH_LINES {
        let mut first = true;
        for m in &mut months {
            if !first {
                out.push_str("   ");
            }
            if let Some(line) = m.pop_front() {
                out.push_str(&line);
            } else {
                out.push_str(&" ".repeat(20));
            }
            first = false;
        }
        if i < MONTH_LINES {
            out.push('\n');
        }
    }
    out
}

fn chunk_months(ctx: &Context, months: &[DateSpec]) -> String {
    months
        .chunks(3)
        .map(|m| merge_months(ctx, m))
        .fold(String::new(), |a, b| {
            if a.is_empty() {
                a + &b
            } else {
                a + "\n" + &b
            }
        })
}

#[derive(Clone)]
enum ActivityGrade {
    None,
    One,
    Two,
    Three,
    Four,
}

impl ActivityGrade {
    fn as_term_str(&self) -> &'static str {
        match self {
            ActivityGrade::None => "\x1b[97m\x1b[48;5;238m",
            ActivityGrade::One => "\x1b[30m\x1b[48;5;148m",
            ActivityGrade::Two => "\x1b[30m\x1b[48;5;71m",
            ActivityGrade::Three => "\x1b[97m\x1b[48;5;34m",
            ActivityGrade::Four => "\x1b[97m\x1b[48;5;22m",
        }
    }
}

struct Context {
    _events: HashMap<Date<Utc>, ActivityGrade>,
    today: Date<Utc>,
}

impl Context {
    fn new() -> Context {
        Context {
            _events: HashMap::new(),
            today: Utc::today(),
        }
    }

    fn is_today(&self, date: &Date<Utc>) -> bool {
        self.today == *date
    }

    fn activity_for_day(&self, date: &Date<Utc>) -> ActivityGrade {
        let _ = date;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 6) {
            0 => ActivityGrade::One,
            1 => ActivityGrade::Two,
            2 => ActivityGrade::Three,
            3 => ActivityGrade::Four,
            _ => ActivityGrade::None,
        }
        /*
        if let Some(activity) = self.events.get(date) {
            activity.clone()
        } else {
            ActivityGrade::None
        }
        */
    }
}

impl DateSpec {
    fn to_term_string(&self, ctx: &Context) -> String {
        match self {
            DateSpec::Year(year) => {
                let months = (1..=12)
                    .map(|month| DateSpec::YearMonth((*year, month)))
                    .collect::<Vec<_>>();
                chunk_months(ctx, &months)
            },
            DateSpec::YearMonth((year, month)) => {
                let mut w = String::new();

                let start = Utc.ymd(*year, *month, 1);
                let days = days_in_month(*year, *month) as u32;

                w.push_str(&format!("{:^21}\n", start.format("%B %Y")));
                w.push_str(" Su Mo Tu We Th Fr Sa\n");

                let mut cur_week_day = start.weekday();
                let week_progress = cur_week_day.num_days_from_sunday() as usize;
                w.push_str(&"   ".repeat(week_progress));

                let mut week_written = week_progress * 3;
                for cur_day in 1..=days {
                    let date = Utc.ymd(*year, *month, cur_day);

                    let activity = ctx.activity_for_day(&date);
                    w.push_str(activity.as_term_str());

                    if ctx.is_today(&date) {
                        w.push_str("\x1b[1m#");
                    } else {
                        w.push(' ');
                    }
                    w.push_str(&format!("{:2}", cur_day));
                    week_written += 3;
                    w.push_str("\x1b[0m");

                    // detect end of the week
                    if cur_week_day == Weekday::Sat {
                        if cur_day != days {
                            w.push('\n');
                        }
                        week_written = 0;
                    }

                    cur_week_day = cur_week_day.succ();
                }
                if week_written != 0 {
                    w.push_str(&" ".repeat(20 - week_written));
                }

                w
            }
            DateSpec::YearMonthContext((year, month, context)) => {
                let mut year = *year - (*context / 12) as i32;
                let mut month = *month - (*context % 12);

                let mut months = Vec::new();

                for _ in 0..=*context {
                    months.push(DateSpec::YearMonth((year, month)));

                    if month == 12 {
                        year += 1;
                        month = 1;
                    } else {
                        month += 1;
                    }
                }

                chunk_months(ctx, &months)
            }
        }
    }
}

impl Cmd for Args {
    #[inline]
    fn run(self, _rl: &mut Shell) -> Result<()> {
        let ds = DateSpec::from_args(&self.args, self.context)
            .context("Failed to parse date spec")?;
        println!("{}", ds.to_term_string(&Context::new()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_in_month_2020_05() {
        let days = days_in_month(2020, 05);
        assert_eq!(days, 31);
    }

    #[test]
    fn test_days_in_month_2020_04() {
        let days = days_in_month(2020, 04);
        assert_eq!(days, 30);
    }

    #[test]
    fn test_days_in_month_2020_02() {
        let days = days_in_month(2020, 02);
        assert_eq!(days, 29);
    }

    #[test]
    fn test_datespec_year_month() {
        let ds = DateSpec::YearMonth((2020, 05));
        let out = ds.to_string();
        assert_eq!(out, "      May 2020      
Su Mo Tu We Th Fr Sa
                1  2
 3  4  5  6  7  8  9
10 11 12 13 14 15 16
17 18 19 20 21 22 23
24 25 26 27 28 29 30
31                  ");
    }

    #[test]
    fn test_datespec_year_month_ends_on_sat() {
        let ds = DateSpec::YearMonth((2020, 10));
        let out = ds.to_string();
        assert_eq!(out, "    October 2020    
Su Mo Tu We Th Fr Sa
             1  2  3
 4  5  6  7  8  9 10
11 12 13 14 15 16 17
18 19 20 21 22 23 24
25 26 27 28 29 30 31");
    }

    #[test]
    fn test_datespec_year() {
        let ds = DateSpec::Year(2020);
        let out = ds.to_string();
        assert_eq!(out, "    January 2020          February 2020            March 2020     
Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa
          1  2  3  4                      1    1  2  3  4  5  6  7
 5  6  7  8  9 10 11    2  3  4  5  6  7  8    8  9 10 11 12 13 14
12 13 14 15 16 17 18    9 10 11 12 13 14 15   15 16 17 18 19 20 21
19 20 21 22 23 24 25   16 17 18 19 20 21 22   22 23 24 25 26 27 28
26 27 28 29 30 31      23 24 25 26 27 28 29   29 30 31            
                                                                  
     April 2020              May 2020              June 2020      
Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa
          1  2  3  4                   1  2       1  2  3  4  5  6
 5  6  7  8  9 10 11    3  4  5  6  7  8  9    7  8  9 10 11 12 13
12 13 14 15 16 17 18   10 11 12 13 14 15 16   14 15 16 17 18 19 20
19 20 21 22 23 24 25   17 18 19 20 21 22 23   21 22 23 24 25 26 27
26 27 28 29 30         24 25 26 27 28 29 30   28 29 30            
                       31                                         
     July 2020             August 2020           September 2020   
Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa
          1  2  3  4                      1          1  2  3  4  5
 5  6  7  8  9 10 11    2  3  4  5  6  7  8    6  7  8  9 10 11 12
12 13 14 15 16 17 18    9 10 11 12 13 14 15   13 14 15 16 17 18 19
19 20 21 22 23 24 25   16 17 18 19 20 21 22   20 21 22 23 24 25 26
26 27 28 29 30 31      23 24 25 26 27 28 29   27 28 29 30         
                       30 31                                      
    October 2020          November 2020          December 2020    
Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa   Su Mo Tu We Th Fr Sa
             1  2  3    1  2  3  4  5  6  7          1  2  3  4  5
 4  5  6  7  8  9 10    8  9 10 11 12 13 14    6  7  8  9 10 11 12
11 12 13 14 15 16 17   15 16 17 18 19 20 21   13 14 15 16 17 18 19
18 19 20 21 22 23 24   22 23 24 25 26 27 28   20 21 22 23 24 25 26
25 26 27 28 29 30 31   29 30                  27 28 29 30 31      
                                                                  ");
    }
}
