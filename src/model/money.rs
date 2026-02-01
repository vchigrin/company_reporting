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

impl ops::AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.roubles += rhs.roubles;
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
}
