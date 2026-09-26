use super::balance_report::BalanceReport;
use super::income_report::IncomeReport;
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

pub struct Report {
    pub balance: BalanceReport,
    pub income: IncomeReport,
}

impl CompanyInfo {
    pub fn build_reports(&self) -> Result<HashMap<Period, Report>> {
        let mut result = HashMap::<Period, Report>::new();
        let parser = report_parser::ReportParser::new();
        for (period, raw_report) in &self.raw_reports {
            let balance = if let Some(lines) = &raw_report.balance {
                parser.parse_balance_report_batch(lines)?
            } else {
                log::warn!("No balance report for {:?}", period);
                continue;
            };
            let income = if let Some(lines) = &raw_report.income {
                parser.parse_income_report_batch(lines)?
            } else {
                log::warn!("No income report for {:?}", period);
                continue;
            };
            result.insert(*period, Report { balance, income });
        }
        Ok(result)
    }
}
