use super::balance_report::BalanceReport;
use super::income_report::IncomeReport;
use super::period::Period;
use std::collections::HashMap;

pub struct Report {
    balance: Option<BalanceReport>,
    income: Option<IncomeReport>,
}

pub struct CompanyInfo {
    name: String,
    reports: HashMap<Period, Report>,
}
