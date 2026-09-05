use eyre::{Result, eyre};
use std::str::FromStr;

#[derive(PartialOrd, PartialEq, Eq, Ord, Copy, Clone, Debug, Hash)]
pub enum PeriodType {
    FirstHalf,
    Full,
}

#[derive(PartialOrd, PartialEq, Eq, Ord, Copy, Clone, Debug, Hash)]
pub struct Period {
    year: i32,
    period_type: PeriodType,
}

impl Period {
    pub fn first_half(year: i32) -> Self {
        Self {
            year,
            period_type: PeriodType::FirstHalf,
        }
    }

    pub fn full(year: i32) -> Self {
        Self {
            year,
            period_type: PeriodType::Full,
        }
    }

    pub fn from_short_string(s: &str) -> Result<Self> {
        if let Some(non_digit) = s.find(|c: char| !c.is_ascii_digit()) {
            let year_str = &s[..non_digit];
            let suffix = &s[non_digit..];
            let year = i32::from_str(year_str)?;
            if suffix == "H1" {
                return Ok(Self::first_half(year));
            } else if suffix == "FULL" {
                return Ok(Self::full(year));
            }
        }
        Err(eyre!("Not parsable period string {}", s))
    }
}
