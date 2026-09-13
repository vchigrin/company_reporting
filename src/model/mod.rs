pub mod balance_report;
mod company;
pub mod income_report;
mod keys;
mod money;
mod period;

pub use company::{CompanyInfo, RawReport};
pub use keys::{BalanceKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
pub use money::Money;
pub use period::Period;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr)]
pub enum ReportType {
    Balance,
    Income,
}

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr)]
pub enum MoneyMultiplier {
    Thousands,
    Millions,
}
