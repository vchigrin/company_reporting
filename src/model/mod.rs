pub mod balance_report;
pub mod income_report;
mod money;
mod period;

pub use money::Money;
pub use period::Period;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, EnumString, IntoStaticStr)]
pub enum ReportType {
    Balance,
    Income,
}

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr)]
pub enum MoneyMultiplier {
    Thousands,
    Millions,
}
