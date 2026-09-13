use super::period::Period;
use crate::report_parser::BalanceKeys;
use crate::report_parser::IncomeKeys;
use crate::report_parser::ParsedLineInfo;
use std::collections::HashMap;

#[derive(Debug)]
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
