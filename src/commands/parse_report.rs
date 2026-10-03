use eyre::{Result, eyre};

use crate::model;
use crate::report_parser;
use crate::report_parser::lines_classifier;
use crate::storage;
use clap::{ArgGroup, Args};
use std::path;
use std::rc::Rc;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
#[command(group(ArgGroup::new("pages").required(true).multiple(false).args(["page_number", "page_from"])))]
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
    page_number: Option<i32>,
    #[arg(long, requires = "page_to")]
    page_from: Option<i32>,
    #[arg(long, requires = "page_from")]
    page_to: Option<i32>,
    #[arg(long, action=clap::ArgAction::SetTrue)]
    interactive: bool,
    #[arg(long)]
    money_multiplier: model::MoneyMultiplier,
    #[arg(long)]
    rsbu: bool,
}

fn classify_balance_lines(
    db: &storage::Storage,
    page_lines: &[String],
    money_multiplier: model::MoneyMultiplier,
    rsbu_mode: bool,
) -> Result<Vec<model::ParsedLineInfo<model::BalanceKeys>>> {
    let dict = db.load_keys_dict::<model::BalanceKeys>(model::ReportType::Balance)?;
    let classifier = report_parser::lines_classifier::LinesClassifier::new(
        money_multiplier,
        Rc::new(lines_classifier::MapKeyClasifier::new(dict)),
    );
    classifier.classify_lines(page_lines, rsbu_mode)
}

fn classify_income_lines(
    db: &storage::Storage,
    page_lines: &[String],
    money_multiplier: model::MoneyMultiplier,
) -> Result<Vec<model::ParsedLineInfo<model::IncomeKeys>>> {
    let dict = db.load_keys_dict::<model::IncomeKeys>(model::ReportType::Income)?;
    let classifier = report_parser::lines_classifier::LinesClassifier::new(
        money_multiplier,
        Rc::new(lines_classifier::MapKeyClasifier::new(dict)),
    );
    // RSBU mode does not any special quicks for income report.
    classifier.classify_lines(page_lines, false)
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

    let page_numbers = match (args.page_number, args.page_from, args.page_to) {
        (Some(page), None, _) => page..=page,
        (None, Some(from), Some(to)) => from..=to,
        _ => {
            panic!("Conflicting page args passed");
        }
    };
    let page_lines = report_parser::get_page_lines(&args.report_path, page_numbers, args.rsbu)?;
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        model::ReportType::Balance => {
            let parsed_lines =
                classify_balance_lines(db, &page_lines, args.money_multiplier, args.rsbu)?;
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
            let parsed_lines = classify_income_lines(db, &page_lines, args.money_multiplier)?;
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
