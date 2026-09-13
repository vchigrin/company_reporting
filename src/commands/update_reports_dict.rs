use eyre::Result;

use crate::model::{self, CompanyInfo, GenericKeys, ParsedLineInfo};
use crate::report_parser::lines_classifier::LinesClassifier;
use crate::storage;
use clap::Args;
use std::collections::HashMap;

#[derive(Debug, Args)]
pub struct UpdateReportsDictArgs {
    #[arg(long, action=clap::ArgAction::SetTrue)]
    force: bool,
}

fn update_dict<Keys: GenericKeys>(
    report_type: model::ReportType,
    db: &mut storage::Storage,
    companies: &[CompanyInfo],
    get_lines: fn(&model::RawReport) -> Option<&Vec<ParsedLineInfo<Keys>>>,
    force: bool,
) -> Result<()> {
    let mut dict: HashMap<String, Keys> = db.load_keys_dict::<Keys>(report_type)?;
    for company in companies {
        for raw_report in company.raw_reports.values() {
            let Some(report_lines) = get_lines(raw_report) else {
                continue;
            };
            for line in report_lines {
                let tokens = LinesClassifier::<Keys>::split_line_to_tokens(&line.original_line);
                if tokens.is_empty() {
                    continue;
                }
                let token = LinesClassifier::<Keys>::get_line_token(&tokens);
                if force {
                    dict.insert(token, line.key);
                } else {
                    dict.entry(token).or_insert(line.key);
                }
            }
        }
    }
    db.overwrite_keys_dict(report_type, &dict)
}

pub fn process_update_reports_dict(
    args: &UpdateReportsDictArgs,
    db: &mut storage::Storage,
) -> Result<()> {
    let companies = db.list_companies()?;
    let force = args.force;

    update_dict::<model::BalanceKeys>(
        model::ReportType::Balance,
        db,
        &companies,
        |r| r.balance.as_ref(),
        force,
    )?;
    update_dict::<model::IncomeKeys>(
        model::ReportType::Income,
        db,
        &companies,
        |r| r.income.as_ref(),
        force,
    )?;

    Ok(())
}
