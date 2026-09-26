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

#[derive(Clone)]
pub struct Report {
    pub balance: BalanceReport,
    pub income: IncomeReport,
}

impl CompanyInfo {
    pub fn build_reports(&self) -> Result<HashMap<Period, Report>> {
        let full_reports = self.build_full_reports()?;
        let uniform_periods = Period::build_uniform(full_reports.keys().copied().collect());
        Ok(Self::build_partial_reports(full_reports, &uniform_periods))
    }

    fn build_partial_reports(
        full_reports: HashMap<Period, Report>,
        periods: &[Period],
    ) -> HashMap<Period, Report> {
        let mut result = HashMap::<Period, Report>::new();
        for period in periods {
            if let Some(ready) = full_reports.get(period) {
                result.insert(*period, ready.clone());
            } else {
                let (full, substracted) = period.make_parts_for_substraction();
                let Some(full_report) = full_reports.get(&full) else {
                    log::warn!(
                        "Can not found period {:?} for building period {:?}",
                        full,
                        period
                    );
                    continue;
                };
                // Substracted can be None only in case when full is equal to
                // |period|. That already handled by logic above.
                let Some(substracted_report) = full_reports.get(&substracted.unwrap()) else {
                    log::warn!(
                        "Can not found period {:?} for building period {:?}",
                        full,
                        period
                    );
                    continue;
                };
                let new_report = Self::substract_reports(full_report, substracted_report);
                result.insert(*period, new_report);
            }
        }
        result
    }

    fn substract_reports(full: &Report, substracted: &Report) -> Report {
        // Balance is the same as at the end of full period.
        let balance = full.balance.clone();
        let income = full.income.clone() - substracted.income.clone();
        Report { balance, income }
    }

    fn build_full_reports(&self) -> Result<HashMap<Period, Report>> {
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
