use eyre::{Result, eyre};

use crate::model;
use crate::report_parser;
use crate::storage;
use clap::{ArgGroup, Args};
use std::path;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct ParseReportArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long, value_parser=model::Period::from_short_string)]
    period: model::Period,
    #[arg(long)]
    report_type: model::ReportType,
    #[arg(long)]
    report_path: path::PathBuf,
    #[arg(long)]
    page_number: i32,
    #[arg(long, action=clap::ArgAction::SetTrue)]
    interactive: bool,
    #[arg(long)]
    money_multiplier: model::MoneyMultiplier,
}

fn classify_balance_lines(
    page_lines: &[String],
    money_multiplier: model::MoneyMultiplier,
) -> Result<Vec<model::ParsedLineInfo<model::BalanceKeys>>> {
    let classifier = report_parser::lines_classifier::LinesClassifier::new(
        money_multiplier,
        report_parser::ReportParser::make_default_balance_keys_classifier(),
    );
    classifier.classify_lines(page_lines)
}

fn classify_income_lines(
    page_lines: &[String],
    money_multiplier: model::MoneyMultiplier,
) -> Result<Vec<model::ParsedLineInfo<model::IncomeKeys>>> {
    let classifier = report_parser::lines_classifier::LinesClassifier::new(
        money_multiplier,
        report_parser::ReportParser::make_default_income_keys_classifier(),
    );
    classifier.classify_lines(page_lines)
}

pub fn process_parse_report(args: &ParseReportArgs, db: &mut storage::Storage) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let company_report: &mut model::RawReport = company.raw_reports.entry(args.period).or_default();
    match args.report_type {
        model::ReportType::Balance => {
            if let Some(existing) = &company_report.balance {
                return Err(eyre!("Balance already present; Value {:?}", existing));
            }
        }
        model::ReportType::Income => {
            if let Some(existing) = &company_report.income {
                return Err(eyre!("Income already present; Value {:?}", existing));
            }
        }
    }

    let page_lines = report_parser::get_page_lines(&args.report_path, args.page_number)?;
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        model::ReportType::Balance => {
            let parsed_lines = classify_balance_lines(&page_lines, args.money_multiplier)?;
            let final_lines = if args.interactive {
                match parser.parse_balance_report_interactive(parsed_lines)? {
                    Some(r) => r,
                    None => {
                        // User cancelled
                        return Ok(());
                    }
                }
            } else {
                // Verify that lines are correct and Income can be constructed
                // from this lines set.
                parser.parse_balance_report_batch(&parsed_lines)?;
                parsed_lines
            };
            company_report.balance = Some(final_lines);
        }
        model::ReportType::Income => {
            let parsed_lines = classify_income_lines(&page_lines, args.money_multiplier)?;
            let final_lines = if args.interactive {
                match parser.parse_income_report_interactive(parsed_lines)? {
                    Some(r) => r,
                    None => {
                        // User cancelled
                        return Ok(());
                    }
                }
            } else {
                // Verify that lines are correct and Income can be constructed
                // from this lines set.
                parser.parse_income_report_batch(&parsed_lines)?;
                parsed_lines
            };
            company_report.income = Some(final_lines);
        }
    }
    db.save_company(&company)?;
    Ok(())
}
