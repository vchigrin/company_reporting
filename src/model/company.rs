use super::keys::{BalanceKeys, IncomeKeys, ParsedLineInfo};
use super::period::Period;
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
