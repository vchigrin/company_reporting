use super::balance_report::BalanceReport;
use super::keys::{BalanceKeys, IncomeKeys, ParsedLineInfo};
use super::period::Period;
use crate::report_parser;
use eyre::Result;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RawReport {
    pub balance: Option<Vec<ParsedLineInfo<BalanceKeys>>>,
    pub income: Option<Vec<ParsedLineInfo<IncomeKeys>>>,
}

#[derive(Debug)]
pub struct CompanyInfo {
    pub name: String,
    pub inn: String,
    pub raw_reports: HashMap<Period, RawReport>,
}

impl CompanyInfo {
    pub fn build_balances(&self) -> Result<HashMap<Period, BalanceReport>> {
        let mut balances = HashMap::<Period, BalanceReport>::new();
        let parser = report_parser::ReportParser::new();
        for (period, raw_report) in &self.raw_reports {
            let Some(balance_lines) = &raw_report.balance else {
                log::info!("Period {}: balance report is absent", period.short_string());
                continue;
            };
            let balance = parser.parse_balance_report_batch(balance_lines)?;
            balances.insert(*period, balance);
        }
        Ok(balances)
    }
}
