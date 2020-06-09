use crate::errors::*;
use std::str::FromStr;

pub mod date;
pub mod time;

#[derive(Debug)]
pub enum DateArg {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityGrade {
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

    pub fn from_ratio(num: u64, max: u64) -> ActivityGrade {
        let max = max as f64;
        let num = num as f64;
        let step = max / 4.0;

        let x = num / step;

        if x <= 1.0 {
            ActivityGrade::One
        } else if x <= 2.0 {
            ActivityGrade::Two
        } else if x <= 3.0 {
            ActivityGrade::Three
        } else {
            ActivityGrade::Four
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_max_activity_1() {
        let grade = ActivityGrade::from_ratio(1, 1);
        assert_eq!(grade, ActivityGrade::Four);
    }

    #[test]
    fn small_max_activity_2() {
        let grade = ActivityGrade::from_ratio(2, 2);
        assert_eq!(grade, ActivityGrade::Four);
    }

    #[test]
    fn small_max_activity_3() {
        let grade = ActivityGrade::from_ratio(3, 3);
        assert_eq!(grade, ActivityGrade::Four);
    }

    #[test]
    fn small_max_activity_4() {
        let grade = ActivityGrade::from_ratio(4, 4);
        assert_eq!(grade, ActivityGrade::Four);
    }

    #[test]
    fn small_max_activity_2_but_is_1() {
        let grade = ActivityGrade::from_ratio(1, 2);
        assert_eq!(grade, ActivityGrade::Two);
    }

    #[test]
    fn small_max_activity_3_but_is_2() {
        let grade = ActivityGrade::from_ratio(2, 3);
        assert_eq!(grade, ActivityGrade::Three);
    }

    #[test]
    fn small_max_activity_3_but_is_1() {
        let grade = ActivityGrade::from_ratio(1, 3);
        assert_eq!(grade, ActivityGrade::Two);
    }

    #[test]
    fn small_max_activity_4_but_is_3() {
        let grade = ActivityGrade::from_ratio(3, 4);
        assert_eq!(grade, ActivityGrade::Three);
    }

    #[test]
    fn small_max_activity_4_but_is_2() {
        let grade = ActivityGrade::from_ratio(2, 4);
        assert_eq!(grade, ActivityGrade::Two);
    }

    #[test]
    fn small_max_activity_4_but_is_1() {
        let grade = ActivityGrade::from_ratio(1, 4);
        assert_eq!(grade, ActivityGrade::One);
    }

    #[test]
    fn small_max_activity_5_but_is_4() {
        let grade = ActivityGrade::from_ratio(4, 5);
        assert_eq!(grade, ActivityGrade::Four);
    }

    #[test]
    fn small_max_activity_5_but_is_3() {
        let grade = ActivityGrade::from_ratio(3, 5);
        assert_eq!(grade, ActivityGrade::Three);
    }

    #[test]
    fn small_max_activity_5_but_is_2() {
        let grade = ActivityGrade::from_ratio(2, 5);
        assert_eq!(grade, ActivityGrade::Two);
    }

    #[test]
    fn small_max_activity_5_but_is_1() {
        let grade = ActivityGrade::from_ratio(1, 5);
        assert_eq!(grade, ActivityGrade::One);
    }
}
