use super::balance_report::BalanceReport;
use super::income_report::IncomeReport;
use super::period::Period;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default)]
pub struct Report {
    pub balance: Option<BalanceReport>,
    pub income: Option<IncomeReport>,
}

#[derive(Debug, PartialEq)]
pub struct CompanyInfo {
    pub name: String,
    pub inn: String,
    pub reports: HashMap<Period, Report>,
}
