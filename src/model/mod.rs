pub mod balance_report;
mod company;
pub mod income_report;
mod keys;
mod money;
mod period;
mod ra_conclusion;

pub use company::{CompanyInfo, RawReport, Report};
pub use keys::{BalanceKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
pub use money::Money;
pub use period::Period;
pub use ra_conclusion::{RAConclusion, Rating, RatingAgency, RatingForecast};
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
