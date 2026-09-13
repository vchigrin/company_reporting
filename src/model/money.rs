use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Money {
    roubles: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print with thousands separator.
        let work_str = self.roubles.abs().to_string();
        if self.roubles < 0 {
            write!(f, "-")?;
        }
        let first_part_digits = work_str.len() % 3;
        if first_part_digits > 0 {
            write!(f, "{} ", &work_str[..first_part_digits])?;
        }
        let mut idx = first_part_digits;
        while idx < work_str.len() {
            write!(f, "{} ", &work_str[idx..(idx + 3)])?;
            idx += 3;
        }
        assert_eq!(idx, work_str.len());
        write!(f, "₽")
    }
}

impl ops::Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            roubles: self.roubles + rhs.roubles,
        }
    }
}

impl ops::AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.roubles += rhs.roubles;
    }
}

impl ops::Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            roubles: self.roubles - rhs.roubles,
        }
    }
}

impl Money {
    pub fn zero() -> Money {
        Self { roubles: 0 }
    }

    pub fn from_thousands(thousands: i64) -> Money {
        Self {
            roubles: thousands * 1000,
        }
    }

    pub fn from_millions(millions: i64) -> Money {
        Self {
            roubles: millions * 1000000,
        }
    }

    pub fn from_roubles(roubles: i64) -> Money {
        Self { roubles }
    }

    pub fn in_roubles(&self) -> i64 {
        self.roubles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Money::zero().to_string(), "0 ₽");

        assert_eq!(Money::from_thousands(2).to_string(), "2 000 ₽");
        assert_eq!(Money::from_thousands(-2).to_string(), "-2 000 ₽");

        assert_eq!(Money::from_thousands(23).to_string(), "23 000 ₽");
        assert_eq!(Money::from_thousands(-23).to_string(), "-23 000 ₽");

        assert_eq!(Money::from_thousands(2345).to_string(), "2 345 000 ₽");
        assert_eq!(Money::from_thousands(-2345).to_string(), "-2 345 000 ₽");

        assert_eq!(
            Money::from_thousands(23456789).to_string(),
            "23 456 789 000 ₽"
        );
        assert_eq!(
            Money::from_thousands(-23456789).to_string(),
            "-23 456 789 000 ₽"
        );
    }
}
