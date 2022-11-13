use crate::errors::*;

use chrono::Utc;
use chrono::prelude::*;
use crate::cal::{ActivityGrade, DateArg};
use crate::models::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Write;

const MONTH_LINES: i32 = 7;

fn merge_months(ctx: &DateContext, months: &[DateSpec]) -> String {
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
                out.push_str(&" ".repeat(21));
            }
            first = false;
        }
        if i < MONTH_LINES {
            out.push('\n');
        }
    }
    out
}

fn chunk_months(ctx: &DateContext, months: &[DateSpec]) -> String {
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

fn days_in_month(year: i32, month: u32) -> i64 {
    let start = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).single().expect("Datetime is not unique");
    let end = if month == 12 {
        Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).single().expect("Datetime is not unique")
    } else {
        Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).single().expect("Datetime is not unique")
    };
    end.signed_duration_since(start).num_days()
}

fn setup_graph_map(events: &[Activity]) -> (HashMap<NaiveDate, u64>, u64) {
    debug!("Found {} events in selected range", events.len());

    let mut cur = None;
    let mut ctr = 0;
    let mut max = 0;

    let mut map = HashMap::new();
    for event in events {
        let date = event.time.date();
        if let Some(cur) = cur.as_mut() {
            if date == *cur {
                ctr += 1;
            } else {
                if ctr > max {
                    max = ctr;
                }
                map.insert(*cur, ctr);
                *cur = date;
                ctr = 1;
            }
        } else {
            cur = Some(date);
            ctr = 1;
        }
    }

    if ctr > 0 {
        if let Some(cur) = cur.take() {
            if ctr > max {
                max = ctr;
            }
            map.insert(cur, ctr);
        }
    }

    debug!("Maximum events per day is {}", max);

    (map, max)
}

pub struct DateContext {
    events: HashMap<NaiveDate, u64>,
    max: u64,
    today: NaiveDate,
}

impl DateContext {
    pub fn new(events: &[Activity], today: NaiveDate) -> DateContext {
        let (events, max) = setup_graph_map(events);
        DateContext {
            events,
            max,
            today,
        }
    }

    #[inline]
    fn is_today(&self, date: &NaiveDate) -> bool {
        self.today == *date
    }

    #[inline]
    fn is_future(&self, date: &NaiveDate) -> bool {
        self.today < *date
    }

    fn activity_for_day(&self, date: &NaiveDate) -> ActivityGrade {
        if let Some(events) = self.events.get(date) {
            ActivityGrade::from_ratio(*events, self.max)
        } else {
            ActivityGrade::None
        }
    }
}

pub enum DateSpec {
    Year(i32),
    YearMonth((i32, u32)),
    YearMonthContext((i32, u32, u32)),
}

impl DateSpec {
    pub fn from_args(args: &[DateArg], context: Option<u32>) -> Result<DateSpec> {
        if args.len() > 2 {
            bail!("Too many datespec args");
        }

        let today = Utc::now();
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

    pub fn start(&self) -> NaiveDate {
        match self {
            DateSpec::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1).expect("Invalid month/day"),
            DateSpec::YearMonth((year, month)) => NaiveDate::from_ymd_opt(*year, *month, 1).expect("Invalid month/day"),
            DateSpec::YearMonthContext((year, month, context)) => {
                let mut year = *year - (*context / 12) as i32;
                let context = context % 12;
                let month = if context >= *month {
                    year -= 1;
                    12 - context + month
                } else {
                    month - context
                };
                NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid month/day")
            },
        }
    }

    pub fn end(&self) -> NaiveDate {
        match self {
            DateSpec::Year(year) => NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("Invalid month/day"),
            DateSpec::YearMonth((year, month)) => {
                let (year, month) = if *month == 12 {
                    (*year + 1, 1)
                } else {
                    (*year, *month + 1)
                };
                NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid month/day")
            },
            DateSpec::YearMonthContext((year, month, _context)) => {
                let (year, month) = if *month == 12 {
                    (*year + 1, 1)
                } else {
                    (*year, *month + 1)
                };
                NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid month/day")
            },
        }
    }

    pub fn to_term_string(&self, ctx: &DateContext) -> String {
        match self {
            DateSpec::Year(year) => {
                let months = (1..=12)
                    .map(|month| DateSpec::YearMonth((*year, month)))
                    .collect::<Vec<_>>();
                chunk_months(ctx, &months)
            },
            DateSpec::YearMonth((year, month)) => {
                let mut w = String::new();

                let start = Utc.with_ymd_and_hms(*year, *month, 1, 0, 0, 0).single().expect("Datetime is not unique");
                let days = days_in_month(*year, *month) as u32;

                writeln!(w, "{:^21}", start.format("%B %Y")).expect("out of memory");
                w.push_str(" Su Mo Tu We Th Fr Sa\n");

                let mut cur_week_day = start.weekday();
                let week_progress = cur_week_day.num_days_from_sunday() as usize;
                w.push_str(&"   ".repeat(week_progress));

                let mut week_written = week_progress * 3;
                for cur_day in 1..=days {
                    let date = NaiveDate::from_ymd_opt(*year, *month, cur_day).expect("Invalid month/day");

                    if !ctx.is_future(&date) {
                        let activity = ctx.activity_for_day(&date);
                        w.push_str(activity.as_term_str());
                    }

                    if ctx.is_today(&date) {
                        w.push_str("\x1b[1m#");
                    } else {
                        w.push(' ');
                    }
                    write!(w, "{:2}", cur_day).expect("out of memory");
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
                    w.push_str(&" ".repeat(21 - week_written));
                }

                w
            }
            DateSpec::YearMonthContext((_year, _month, context)) => {
                let start = self.start();
                let mut year = start.year();
                let mut month = start.month();

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

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> DateContext {
        DateContext {
            events: HashMap::new(),
            max: 0,
            today: NaiveDate::from_ymd_opt(2020, 5, 30).unwrap(),
        }
    }

    #[test]
    fn test_days_in_month_2020_05() {
        let days = days_in_month(2020, 5);
        assert_eq!(days, 31);
    }

    #[test]
    fn test_days_in_month_2020_04() {
        let days = days_in_month(2020, 4);
        assert_eq!(days, 30);
    }

    #[test]
    fn test_days_in_month_2020_02() {
        let days = days_in_month(2020, 2);
        assert_eq!(days, 29);
    }

    #[test]
    fn small_max_activity_0() {
        let events = HashMap::new();
        let ctx = DateContext {
            events,
            max: 0,
            today: NaiveDate::from_ymd_opt(2020, 6, 6).unwrap(),
        };
        let grade = ctx.activity_for_day(&NaiveDate::from_ymd_opt(2020, 6, 6).unwrap());
        assert_eq!(grade, ActivityGrade::None);
    }

    #[test]
    fn test_datespec_year_month() {
        let ds = DateSpec::YearMonth((2020, 5));
        let out = ds.to_term_string(&context());
        assert_eq!(out, "      May 2020       
 Su Mo Tu We Th Fr Sa
               \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m\u{1b}[1m#30\u{1b}[0m
 31\u{1b}[0m                  ");
    }

    #[test]
    fn test_datespec_year_month_ends_on_sat() {
        let ds = DateSpec::YearMonth((2020, 10));
        let out = ds.to_term_string(&context());
        assert_eq!(out, "    October 2020     
 Su Mo Tu We Th Fr Sa
              1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m
  4\u{1b}[0m  5\u{1b}[0m  6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m
 11\u{1b}[0m 12\u{1b}[0m 13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m
 18\u{1b}[0m 19\u{1b}[0m 20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m
 25\u{1b}[0m 26\u{1b}[0m 27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m 31\u{1b}[0m");
    }

    #[test]
    fn test_datespec_year() {
        let ds = DateSpec::Year(2020);
        let out = ds.to_term_string(&context());
        assert_eq!(out, "    January 2020            February 2020            March 2020      
 Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa
         \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m                     \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 30\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 31\u{1b}[0m      \u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 30\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 31\u{1b}[0m            
                                                                     
     April 2020               May 2020                June 2020      
 Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa
         \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m                  \u{1b}[97m\u{1b}[48;5;238m  1\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  2\u{1b}[0m        1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m  5\u{1b}[0m  6\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m  3\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  4\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  5\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  6\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  7\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  8\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m  9\u{1b}[0m     7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m 12\u{1b}[0m 13\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 10\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 11\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 12\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 13\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 14\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 15\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 16\u{1b}[0m    14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m 19\u{1b}[0m 20\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m   \u{1b}[97m\u{1b}[48;5;238m 17\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 18\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 19\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 20\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 21\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 22\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 23\u{1b}[0m    21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m 26\u{1b}[0m 27\u{1b}[0m
\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 30\u{1b}[0m         \u{1b}[97m\u{1b}[48;5;238m 24\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 25\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 26\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 27\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 28\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m 29\u{1b}[0m\u{1b}[97m\u{1b}[48;5;238m\u{1b}[1m#30\u{1b}[0m    28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m            
                         31\u{1b}[0m                                          
      July 2020              August 2020           September 2020    
 Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa
           1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m                       1\u{1b}[0m           1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m  5\u{1b}[0m
  5\u{1b}[0m  6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m     2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m  5\u{1b}[0m  6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m     6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m 12\u{1b}[0m
 12\u{1b}[0m 13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m     9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m 12\u{1b}[0m 13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m    13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m 19\u{1b}[0m
 19\u{1b}[0m 20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m    16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m 19\u{1b}[0m 20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m    20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m 26\u{1b}[0m
 26\u{1b}[0m 27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m 31\u{1b}[0m       23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m 26\u{1b}[0m 27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m    27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m         
                         30\u{1b}[0m 31\u{1b}[0m                                       
    October 2020            November 2020           December 2020    
 Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa    Su Mo Tu We Th Fr Sa
              1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m     1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m  5\u{1b}[0m  6\u{1b}[0m  7\u{1b}[0m           1\u{1b}[0m  2\u{1b}[0m  3\u{1b}[0m  4\u{1b}[0m  5\u{1b}[0m
  4\u{1b}[0m  5\u{1b}[0m  6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m     8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m 12\u{1b}[0m 13\u{1b}[0m 14\u{1b}[0m     6\u{1b}[0m  7\u{1b}[0m  8\u{1b}[0m  9\u{1b}[0m 10\u{1b}[0m 11\u{1b}[0m 12\u{1b}[0m
 11\u{1b}[0m 12\u{1b}[0m 13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m    15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m 19\u{1b}[0m 20\u{1b}[0m 21\u{1b}[0m    13\u{1b}[0m 14\u{1b}[0m 15\u{1b}[0m 16\u{1b}[0m 17\u{1b}[0m 18\u{1b}[0m 19\u{1b}[0m
 18\u{1b}[0m 19\u{1b}[0m 20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m    22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m 26\u{1b}[0m 27\u{1b}[0m 28\u{1b}[0m    20\u{1b}[0m 21\u{1b}[0m 22\u{1b}[0m 23\u{1b}[0m 24\u{1b}[0m 25\u{1b}[0m 26\u{1b}[0m
 25\u{1b}[0m 26\u{1b}[0m 27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m 31\u{1b}[0m    29\u{1b}[0m 30\u{1b}[0m                   27\u{1b}[0m 28\u{1b}[0m 29\u{1b}[0m 30\u{1b}[0m 31\u{1b}[0m      
                                                                     ");
    }
}
