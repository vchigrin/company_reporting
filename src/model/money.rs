use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    roubles: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ₽", self.roubles)
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
