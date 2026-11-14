use super::balance_report::BalanceReport;
use super::income_report::IncomeReport;
use super::keys::{BalanceKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
use super::note::Note;
use super::period::Period;
use super::ra_conclusion::RAConclusion;
use crate::report_parser;
use eyre::Result;
use std::collections::HashMap;
use strum_macros::{EnumString, IntoStaticStr, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantArray, IntoStaticStr)]
pub enum ReportStandard {
    Rsbu,
    Msfo,
}

#[derive(Debug)]
pub struct RawReport<Keys: GenericKeys> {
    pub lines: Vec<ParsedLineInfo<Keys>>,
    pub standard: ReportStandard,
}

#[derive(Debug, Default)]
pub struct ReportSet {
    pub balance: Option<RawReport<BalanceKeys>>,
    pub income: Option<RawReport<IncomeKeys>>,
}

#[derive(Debug)]
pub struct CompanyInfo {
    pub name: String,
    pub inn: String,
    pub raw_reports: HashMap<Period, ReportSet>,
    pub ra_conclusions: Vec<RAConclusion>,
    pub notes: Vec<Note>,
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
        for (period, report_set) in &self.raw_reports {
            let balance = if let Some(b) = &report_set.balance {
                parser.parse_balance_report_batch(b)?
            } else {
                log::warn!("No balance report for {:?}", period);
                continue;
            };
            let income = if let Some(i) = &report_set.income {
                parser.parse_income_report_batch(i)?
            } else {
                log::warn!("No income report for {:?}", period);
                continue;
            };
            result.insert(*period, Report { balance, income });
        }
        Ok(result)
    }
}
