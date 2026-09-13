use clap::{Args, Parser, Subcommand};
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::path;

mod model;
mod report_parser;
mod storage;

// TODO(vchigrin): Add config or command line param or path in HOME directory...
const DB_FILE_PATH: &str = "companies.db";

#[derive(Debug, Args)]
struct AddCompanyArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    inn: String,
}

#[derive(Debug, Args)]
struct GetCompanyArgs {
    #[arg(long)]
    inn: String,
}

#[derive(Debug, Args)]
struct ParseReportArgs {
    #[arg(long)]
    company_inn: String,
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

#[derive(Debug, Subcommand)]
enum Command {
    AddCompany(AddCompanyArgs),
    GetCompany(GetCompanyArgs),
    ListCompanies,
    ParseReport(ParseReportArgs),
}

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Tool for parsing and evaluation company financial reports"
)]
struct CliParams {
    #[command(subcommand)]
    command: Command,
}

fn process_add_company(args: &AddCompanyArgs) -> Result<()> {
    let mut db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
    let company = model::CompanyInfo {
        name: args.name.clone(),
        inn: args.inn.clone(),
        raw_reports: HashMap::default(),
    };
    db.save_company(&company)?;
    println!("Saved company {:?}", company);
    Ok(())
}

fn process_get_company(args: &GetCompanyArgs) -> Result<()> {
    let db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
    let company = db.get_company_by_inn(&args.inn)?;
    println!("Company: {} INN: {}", company.name, company.inn);
    let mut periods: Vec<_> = company.raw_reports.keys().collect();
    periods.sort();
    println!("Known reports:");
    for period in periods {
        let report = &company.raw_reports[period];
        let has_balance = report.balance.is_some();
        let has_income = report.income.is_some();
        println!(
            "    {}; Has balance? {}; Has income report? {}",
            period.short_string(),
            has_balance,
            has_income
        );
    }
    Ok(())
}

fn process_parse_report(args: &ParseReportArgs) -> Result<()> {
    let mut db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
    let mut company = db.get_company_by_inn(&args.company_inn)?;
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
            let parsed_lines = parser.classify_balance_lines(&page_lines, args.money_multiplier)?;
            let final_lines = if args.interactive {
                parser.parse_balance_report_interactive(parsed_lines)?
            } else {
                // Verify that lines are correct and Income can be constructed
                // from this lines set.
                parser.parse_balance_report_batch(&parsed_lines)?;
                parsed_lines
            };
            company_report.balance = Some(final_lines);
        }
        model::ReportType::Income => {
            let parsed_lines = parser.classify_income_lines(&page_lines, args.money_multiplier)?;
            let final_lines = if args.interactive {
                parser.parse_income_report_interactive(parsed_lines)?
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

fn process_list_companies() -> Result<()> {
    let db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
    for company in db.list_companies()? {
        println!("Company: {} INN: {}", company.name, company.inn);
    }
    Ok(())
}

fn process_command(command: &Command) -> Result<()> {
    match command {
        Command::AddCompany(args) => process_add_company(args),
        Command::GetCompany(args) => process_get_company(args),
        Command::ListCompanies => process_list_companies(),
        Command::ParseReport(args) => process_parse_report(args),
    }
}

fn do_main() -> Result<()> {
    let params = CliParams::parse();
    println!("Running with params {:?}", params);
    process_command(&params.command)?;
    Ok(())
}

fn main() {
    simple_logging::log_to_file("company_reporting.log", log::LevelFilter::Debug).unwrap();
    if let Err(e) = do_main() {
        println!("Error {}", e);
        std::process::exit(1);
    }
}
