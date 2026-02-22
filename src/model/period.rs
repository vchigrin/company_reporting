use eyre::{Result, eyre};
use std::str::FromStr;

#[derive(PartialOrd, PartialEq, Eq, Ord, Copy, Clone, Debug, Hash)]
pub enum PeriodType {
    FirstHalf,
    // Non-standard, constructed from Full and FirstHalf periods.
    SecondHalf,
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

    pub fn second_half(year: i32) -> Self {
        Self {
            year,
            period_type: PeriodType::SecondHalf,
        }
    }

    pub fn full(year: i32) -> Self {
        Self {
            year,
            period_type: PeriodType::Full,
        }
    }

    pub fn period_type(&self) -> PeriodType {
        self.period_type
    }

    pub fn short_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.year.to_string());
        match self.period_type {
            PeriodType::FirstHalf => {
                result += "H1";
            }
            PeriodType::SecondHalf => {
                result += "H2";
            }
            PeriodType::Full => {
                result += "FULL";
            }
        }
        result
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
