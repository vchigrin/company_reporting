use eyre::{Result, eyre};

use crate::model;
use crate::model::ReportType;
use crate::report_parser;
use crate::storage;
use clap::{ArgGroup, Args};

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct EditReportArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long, value_parser=model::Period::from_short_string)]
    period: model::Period,
    #[arg(long)]
    report_type: model::ReportType,
}

pub fn process_edit_report(args: &EditReportArgs, db: &mut storage::Storage) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let company_report = if let Some(m) = company.raw_reports.get_mut(&args.period) {
        m
    } else {
        return Err(eyre!("No report for this period found"));
    };
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        ReportType::Balance => {
            let existing = company_report
                .balance
                .take()
                .ok_or_else(|| eyre!("No balance report for this period to edit"))?;
            let edited = parser.parse_balance_report_interactive(existing)?;
            company_report.balance = Some(edited);
        }
        ReportType::Income => {
            let existing = company_report
                .income
                .take()
                .ok_or_else(|| eyre!("No income report for this period to edit"))?;
            let edited = parser.parse_income_report_interactive(existing)?;
            company_report.income = Some(edited);
        }
    }
    db.save_company(&company)?;
    Ok(())
}
