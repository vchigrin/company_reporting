pub mod balance_report;
mod company;
pub mod income_report;
mod money;
mod period;

pub use company::{CompanyInfo, Report};
pub use money::Money;
pub use period::Period;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, EnumString, IntoStaticStr)]
pub enum ReportType {
    Balance,
    Income,
}
