use eyre::{Result, eyre};
use std::str::FromStr;

#[derive(PartialOrd, PartialEq, Eq, Ord, Copy, Clone, Debug, Hash)]
pub enum PeriodType {
    FirstHalf,
    // Artifically constructed from "Full" and "FirstHalf" reports.
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

    fn is_half(&self) -> bool {
        self.period_type == PeriodType::FirstHalf || self.period_type == PeriodType::SecondHalf
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
            } else if suffix == "H2" {
                return Ok(Self::second_half(year));
            } else if suffix == "FULL" {
                return Ok(Self::full(year));
            }
        }
        Err(eyre!("Not parsable period string {}", s))
    }

    // Splits bigger periods to made periods of the same "size"
    pub fn build_uniform(periods: Vec<Period>) -> Vec<Period> {
        let has_only_full = periods.iter().all(|k| k.period_type == PeriodType::Full);
        if has_only_full {
            return periods;
        }

        // Unlikely situation, useful to handle if we have just one report
        // for half-year.
        let has_only_halves = periods.iter().all(|k| k.is_half());
        if has_only_halves {
            return periods;
        }
        // TODO(vchigrin): Here logic must be extended when we'll add quarter reports.
        let mut result = Vec::new();
        for period in &periods {
            if period.is_half() {
                result.push(*period);
            } else {
                result.push(Period {
                    year: period.year,
                    period_type: PeriodType::FirstHalf,
                });
                result.push(Period {
                    year: period.year,
                    period_type: PeriodType::SecondHalf,
                });
            }
        }
        result.sort();
        result.dedup();
        result
    }

    pub fn make_parts_for_substraction(self) -> (Period, Option<Period>) {
        match self.period_type {
            PeriodType::FirstHalf => (self, None),
            PeriodType::Full => (self, None),
            PeriodType::SecondHalf => {
                let full = Period {
                    year: self.year,
                    period_type: PeriodType::Full,
                };
                let substracted = Period {
                    year: self.year,
                    period_type: PeriodType::FirstHalf,
                };
                (full, Some(substracted))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_uniform_empty() {
        assert_eq!(Period::build_uniform(vec![]), vec![]);
    }

    #[test]
    fn build_uniform_full_only_kept_as_is() {
        let periods = vec![Period::full(2020), Period::full(2021)];
        assert_eq!(Period::build_uniform(periods.clone()), periods);

        let single = vec![Period::full(2020)];
        assert_eq!(Period::build_uniform(single.clone()), single);
    }

    #[test]
    fn build_uniform_halves_only_kept_as_is() {
        let periods = vec![Period::first_half(2020), Period::second_half(2020)];
        assert_eq!(Period::build_uniform(periods.clone()), periods);
    }

    #[test]
    fn build_uniform_full_split_into_halves_when_mixed() {
        let periods = vec![Period::full(2020), Period::first_half(2021)];
        let expected = vec![
            Period::first_half(2020),
            Period::second_half(2020),
            Period::first_half(2021),
        ];
        assert_eq!(Period::build_uniform(periods), expected);
    }

    #[test]
    fn build_uniform_mixed_full_and_halves() {
        let periods = vec![
            Period::full(2020),
            Period::first_half(2020),
            Period::second_half(2021),
        ];
        let expected = vec![
            Period::first_half(2020),
            Period::second_half(2020),
            Period::second_half(2021),
        ];
        assert_eq!(Period::build_uniform(periods), expected);
    }

    #[test]
    fn build_uniform_dedups_duplicates() {
        let periods = vec![
            Period::full(2020),
            Period::first_half(2020),
            Period::full(2020),
        ];
        let expected = vec![Period::first_half(2020), Period::second_half(2020)];
        assert_eq!(Period::build_uniform(periods), expected);
    }

    #[test]
    fn make_parts_for_substraction_first_half() {
        let half = Period::first_half(2021);
        assert_eq!(half.make_parts_for_substraction(), (half, None));
    }

    #[test]
    fn make_parts_for_substraction_full() {
        let full = Period::full(2021);
        assert_eq!(full.make_parts_for_substraction(), (full, None));
    }

    #[test]
    fn make_parts_for_substraction_second_half() {
        let half = Period::second_half(2021);
        let expected = (Period::full(2021), Some(Period::first_half(2021)));
        assert_eq!(half.make_parts_for_substraction(), expected);
    }
}
