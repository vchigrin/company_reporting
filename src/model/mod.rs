pub mod balance_report;
pub mod cash_flow_report;
mod company;
pub mod income_report;
mod keys;
mod money;
mod note;
mod period;
mod ra_conclusion;

pub use company::{CompanyInfo, RawReport, Report, ReportSet, ReportStandard};
pub use keys::{BalanceKeys, CashFlowKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
pub use money::Money;
pub use note::Note;
pub use period::Period;
pub use ra_conclusion::{RAConclusion, Rating, RatingAgency, RatingForecast};
use strum_macros::{EnumString, IntoStaticStr, VariantArray};

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr, VariantArray)]
pub enum ReportType {
    Balance,
    Income,
    CashFlow,
}

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr, VariantArray)]
pub enum MoneyMultiplier {
    Thousands,
    Millions,
}
