use crate::errors::*;

use chrono::Duration;
use chrono::prelude::*;
use crate::cal::{ActivityGrade, DateArg};
use crate::models::*;
use std::collections::HashMap;
use std::fmt::Write;

const MIN_PER_DAY: u32 = 1440;

fn round_to_slice(time: &NaiveDateTime, slice_duration: u32) -> NaiveDateTime {
    let date = time.date();
    let hour = time.hour();
    let mins = time.minute();
    let slice = mins - (mins % slice_duration);
    date.and_hms_opt(hour, slice, 0).expect("Invalid hour/min/sec")
}

fn setup_graph_map(events: &[Activity], slice_duration: u32) -> (HashMap<NaiveDateTime, u64>, u64) {
    debug!("Found {} events in selected range", events.len());

    let mut cur = None;
    let mut ctr = 0;
    let mut max = 0;

    let mut map = HashMap::new();
    for event in events {
        let time = round_to_slice(&event.time, slice_duration);

        if let Some(cur) = cur.as_mut() {
            if time == *cur {
                ctr += 1;
            } else {
                if ctr > max {
                    max = ctr;
                }
                map.insert(*cur, ctr);
                *cur = time;
                ctr = 1;
            }
        } else {
            cur = Some(time);
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

    debug!("Maximum events per slice is {}", max);

    (map, max)
}

pub struct DateTimeContext {
    events: HashMap<NaiveDateTime, u64>,
    max: u64,
    now: NaiveDateTime,
    pub slice_width: u32,
    pub slice_duration: u32,
}

impl DateTimeContext {
    pub fn new(events: &[Activity], now: NaiveDateTime, slice_width: u32, slice_duration: u32) -> DateTimeContext {
        let (events, max) = setup_graph_map(events, slice_duration);
        DateTimeContext {
            events,
            max,
            now: round_to_slice(&now, slice_duration),
            slice_width,
            slice_duration,
        }
    }

    #[inline]
    fn is_future(&self, time: &NaiveDateTime) -> bool {
        self.now < *time
    }

    #[inline]
    fn slices_per_hour(&self) -> u32 {
        60 / self.slice_duration
    }

    #[inline]
    fn hour_width(&self) -> u32 {
        self.slices_per_hour() * self.slice_width
    }

    fn activity_for_slice(&self, time: &NaiveDateTime) -> ActivityGrade {
        if let Some(events) = self.events.get(time) {
            ActivityGrade::from_ratio(*events, self.max)
        } else {
            ActivityGrade::None
        }
    }
}

pub struct DateTimeSpec {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateTimeSpec {
    pub fn from_args(args: &[DateArg], context: Option<u32>) -> Result<DateTimeSpec> {
        let today = Utc::now().date_naive();
        if args.is_empty() {
            let mut start = today;

            if let Some(context) = context {
                start = start.checked_sub_signed(Duration::days(context as i64))
                    .ok_or_else(|| format_err!("Failed travel back in time"))?;
            }

            Ok(DateTimeSpec {
                start,
                end: today,
            })
        } else {
            todo!()
        }
    }

    pub fn start(&self) -> &NaiveDate {
        &self.start
    }

    pub fn end(&self) -> &NaiveDate {
        &self.end
    }

    fn push_date(&self, w: &mut String, date: &NaiveDate) {
        let weekday = date.weekday();
        let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;

        if is_weekend {
            w.push_str("\x1b[1m");
        }

        w.push_str(&date.format("%Y-%m-%d (%a) ").to_string());

        if is_weekend {
            w.push_str("\x1b[0m");
        }
    }

    pub fn to_term_string(&self, ctx: &DateTimeContext) -> String {
        let mut w = String::new();

        // add legend
        w.push_str(&" ".repeat(11));
        for x in 0..24 {
            write!(w, "{:02}", x).expect("out of memory");

            for i in 0..ctx.hour_width() {
                if i >= 2 {
                    w.push(' ');
                }
            }
        }
        w.push('\n');

        // add days
        let mut date = self.start;
        while date <= self.end {
            self.push_date(&mut w, &date);

            let mut hours = 0;
            let mut mins = 0;

            for _ in 0..(MIN_PER_DAY / ctx.slice_duration) {
                let time = date.and_hms_opt(hours, mins, 0).expect("Invalid hour/min/sec");

                if !ctx.is_future(&time) {
                    let activity = ctx.activity_for_slice(&time);
                    w.push_str(activity.as_term_str());
                } else {
                    w.push_str("\x1b[0m");
                }

                for _ in 0..ctx.slice_width {
                    w.push(' ');
                }

                mins += ctx.slice_duration;
                if mins >= 60 {
                    hours += 1;
                    mins = 0;
                }
            }
            w.push_str("\x1b[0m");

            if date < self.end {
                w.push('\n');
            }

            date = date.checked_add_signed(Duration::days(1))
                .expect("Failed to get next day");
        }

        w
    }
}
