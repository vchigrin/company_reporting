use crate::model::balance_report::BalanceReport;
use crate::model::income_report::IncomeReport;
use eyre::Result;

pub fn parse_balance_report(page_lines: &[String]) -> Result<BalanceReport> {
    println!("Got lines {:?}", page_lines);
    unimplemented!();
}

pub fn parse_income_report(page_lines: &[String]) -> Result<IncomeReport> {
    unimplemented!();
}
