use crate::errors::*;

use chrono::Utc;
use chrono::prelude::*;
use crate::cmd::Cmd;
// use crate::config::Config;
// use crate::db::Database;
use crate::shell::Shell;
// use crate::term;
// use crate::utils;
use std::collections::VecDeque;
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;
use structopt::clap::AppSettings;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
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
    // YearMonthContext((i32, u32, u8)),
}

impl DateSpec {
    fn from_args(args: &[DateArg]) -> Result<DateSpec> {
        if args.len() > 2 {
            bail!("Too many datespec args");
        }

        let today = Utc::today();
        let ds = match (args.get(0), args.get(1)) {
            (None, _) => DateSpec::YearMonth((today.year(), today.month())),
            (Some(DateArg::Month(month)), None) => DateSpec::YearMonth((today.year(), *month)),
            (Some(DateArg::Num(year)), None) => DateSpec::Year(*year),
            (Some(DateArg::Month(month)), Some(DateArg::Num(year))) => DateSpec::YearMonth((*year, *month)),
            (Some(DateArg::Num(year)), Some(DateArg::Month(month))) => DateSpec::YearMonth((*year, *month)),
            _ => bail!("Combination of datespec args is invalid"),
        };
        Ok(ds)
    }
}

const MONTH_LINES: i32 = 7;

fn merge_months(months: &[DateSpec]) -> String {
    let mut months = months.iter()
        .map(|ds| {
            let month = ds.to_string();
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

impl fmt::Display for DateSpec {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateSpec::Year(year) => {
                let months = (1..=12)
                    .map(|month| DateSpec::YearMonth((*year, month)))
                    .collect::<Vec<_>>();
                let out = months
                    .chunks(3)
                    .map(merge_months)
                    .fold(String::new(), |a, b| {
                        if a.is_empty() {
                            a + &b
                        } else {
                            a + "\n" + &b
                        }
                    });
                write!(w, "{}", out)?;
                Ok(())
            },
            DateSpec::YearMonth((year, month)) => {
                let start = Utc.ymd(*year, *month, 1);
                let days = days_in_month(*year, *month);

                write!(w, "{:^20}\n", start.format("%B %Y"))?;
                write!(w, "Su Mo Tu We Th Fr Sa\n")?;

                let mut cur_week_day = start.weekday();
                let week_progress = cur_week_day.num_days_from_sunday() as usize;
                write!(w, "{}", "   ".repeat(week_progress))?;

                let mut week_written = week_progress * 3;
                for cur_day in 1..=days {
                    write!(w, "{:2}", cur_day)?;
                        week_written += 2;

                    // detect end of the week
                    if cur_week_day == Weekday::Sat {
                        if cur_day != days {
                            write!(w, "\n")?;
                        }
                        week_written = 0;
                    } else {
                        write!(w, " ")?;
                        week_written += 1;
                    }

                    cur_week_day = cur_week_day.succ();
                }
                if week_written != 0 {
                    write!(w, "{}", " ".repeat(20 - week_written))?;
                }

                Ok(())
            }
            // DateSpec::YearMonthContext((year, month, context)) => todo!(),
        }
    }
}

impl Cmd for Args {
    #[inline]
    fn run(self, _rl: &mut Shell) -> Result<()> {
        let ds = DateSpec::from_args(&self.args)
            .context("Failed to parse date spec")?;
        println!("{}", ds);
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
