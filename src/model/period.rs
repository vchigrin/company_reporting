use eyre::{Result, eyre};
use std::str::FromStr;

#[derive(PartialOrd, PartialEq, Eq, Ord, Copy, Clone, Debug, Hash)]
pub enum PeriodType {
    FirstHalf,
    // Artifically constructed from "Full" and "FirstHalf" reports.
    SecondHalf,
    Q1,
    Q2,
    Q3,
    Q4,
    // First 3 quarters.
    Q13,
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

    fn is_quarter(&self) -> bool {
        self.period_type == PeriodType::Q1
            || self.period_type == PeriodType::Q2
            || self.period_type == PeriodType::Q3
            || self.period_type == PeriodType::Q4
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
            PeriodType::Q1 => {
                result += "Q1";
            }
            PeriodType::Q2 => {
                result += "Q2";
            }
            PeriodType::Q3 => {
                result += "Q3";
            }
            PeriodType::Q4 => {
                result += "Q4";
            }
            PeriodType::Q13 => {
                result += "Q13";
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
            return match suffix {
                "H1" => Ok(Self::first_half(year)),
                "H2" => Ok(Self::second_half(year)),
                "FULL" => Ok(Self::full(year)),
                "Q1" => Ok(Self {
                    year,
                    period_type: PeriodType::Q1,
                }),
                "Q2" => Ok(Self {
                    year,
                    period_type: PeriodType::Q2,
                }),
                "Q3" => Ok(Self {
                    year,
                    period_type: PeriodType::Q3,
                }),
                "Q4" => Ok(Self {
                    year,
                    period_type: PeriodType::Q4,
                }),
                "Q13" => Ok(Self {
                    year,
                    period_type: PeriodType::Q13,
                }),
                _ => Err(eyre!("Not parsable period suffix {}", s)),
            };
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
        let has_only_quarters = periods.iter().all(|k| k.is_quarter());
        if has_only_quarters {
            return periods;
        }
        let mut result = Vec::new();
        let has_any_quarters = periods
            .iter()
            .any(|k| k.is_quarter() || k.period_type == PeriodType::Q13);
        if has_any_quarters {
            // Need split up to quarter level.
            for period in &periods {
                result.extend(period.to_quarters());
            }
        } else {
            // Need handle only halves
            for period in &periods {
                result.extend(period.to_halves());
            }
        }
        result.sort();
        result.dedup();
        result
    }

    fn to_halves(self) -> Vec<Period> {
        if self.is_half() {
            vec![self]
        } else {
            vec![
                Period {
                    year: self.year,
                    period_type: PeriodType::FirstHalf,
                },
                Period {
                    year: self.year,
                    period_type: PeriodType::SecondHalf,
                },
            ]
        }
    }

    fn to_quarters(self) -> Vec<Period> {
        match self.period_type {
            PeriodType::FirstHalf => {
                vec![
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                ]
            }
            PeriodType::SecondHalf => {
                vec![
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q3,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q4,
                    },
                ]
            }
            PeriodType::Q1 => {
                vec![self]
            }
            PeriodType::Q2 => {
                vec![self]
            }
            PeriodType::Q3 => {
                vec![self]
            }
            PeriodType::Q4 => {
                vec![self]
            }
            PeriodType::Q13 => {
                vec![
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q3,
                    },
                ]
            }
            PeriodType::Full => {
                vec![
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q3,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q4,
                    },
                ]
            }
        }
    }

    // In reports we can encounter only periods of type
    // FULL, Q1, FirstHalf, Q13.
    // This methods allows us build any period from them -
    // returns two periods, substraction of which produces self period.
    // If this period does not need substraction, then second returned
    // item is None.
    pub fn make_parts_for_substraction(self) -> (Period, Option<Period>) {
        match self.period_type {
            PeriodType::FirstHalf => (self, None),
            PeriodType::Full => (self, None),
            PeriodType::Q1 => (self, None),
            PeriodType::Q13 => (self, None),
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
            PeriodType::Q2 => {
                let full = Period {
                    year: self.year,
                    period_type: PeriodType::FirstHalf,
                };
                let substracted = Period {
                    year: self.year,
                    period_type: PeriodType::Q1,
                };
                (full, Some(substracted))
            }
            PeriodType::Q3 => {
                let full = Period {
                    year: self.year,
                    period_type: PeriodType::Q13,
                };
                let substracted = Period {
                    year: self.year,
                    period_type: PeriodType::FirstHalf,
                };
                (full, Some(substracted))
            }
            PeriodType::Q4 => {
                let full = Period {
                    year: self.year,
                    period_type: PeriodType::Full,
                };
                let substracted = Period {
                    year: self.year,
                    period_type: PeriodType::Q13,
                };
                (full, Some(substracted))
            }
        }
    }

    // Returns periods for last year, including current period.
    // Uses same granularity as current period (e.g. if current period is
    // Q2, then returns four periods for quarters, rather then two halves.
    pub fn get_periods_for_ltm(self) -> Vec<Period> {
        match self.period_type {
            PeriodType::FirstHalf => {
                let prev = Period {
                    year: self.year - 1,
                    period_type: PeriodType::SecondHalf,
                };
                vec![prev, self]
            }
            PeriodType::Full => vec![self],
            PeriodType::SecondHalf => {
                let prev = Period {
                    year: self.year,
                    period_type: PeriodType::FirstHalf,
                };
                vec![prev, self]
            }
            PeriodType::Q1 => {
                vec![
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q2,
                    },
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q3,
                    },
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q4,
                    },
                    self,
                ]
            }
            PeriodType::Q2 => {
                vec![
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q3,
                    },
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q4,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    self,
                ]
            }
            PeriodType::Q3 => {
                vec![
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q4,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                    self,
                ]
            }
            PeriodType::Q4 => {
                vec![
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q3,
                    },
                    self,
                ]
            }
            PeriodType::Q13 => {
                vec![
                    Period {
                        year: self.year - 1,
                        period_type: PeriodType::Q4,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q1,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q2,
                    },
                    Period {
                        year: self.year,
                        period_type: PeriodType::Q3,
                    },
                ]
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
