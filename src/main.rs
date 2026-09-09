use clap::{Args, Parser, Subcommand};
use eyre::Result;
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
struct ParseReportArgs {
    #[arg(long)]
    company_name: String,
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
    let db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
    let company = model::CompanyInfo {
        name: args.name.clone(),
        inn: args.inn.clone(),
    };
    db.save_company(&company)?;
    println!("Saved company {:?}", company);
    Ok(())
}

fn process_parse_report(args: &ParseReportArgs) -> Result<()> {
    let page_lines = report_parser::get_page_lines(&args.report_path, args.page_number)?;
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        model::ReportType::Balance => {
            let parsed_lines = parser.classify_balance_lines(&page_lines, args.money_multiplier)?;
            let report = if args.interactive {
                parser.parse_balance_report_interactive(parsed_lines)?
            } else {
                parser.parse_balance_report_batch(parsed_lines)?
            };
            println!("Parsed balance {:?}", report);
            // TODO: save to DB.
        }
        model::ReportType::Income => {
            let parsed_lines = parser.classify_income_lines(&page_lines, args.money_multiplier)?;
            let report = if args.interactive {
                parser.parse_income_report_interactive(parsed_lines)?
            } else {
                parser.parse_income_report_batch(parsed_lines)?
            };
            println!("Parsed income {:?}", report);
        }
    }
    Ok(())
}

fn process_command(command: &Command) -> Result<()> {
    match command {
        Command::AddCompany(args) => process_add_company(args),
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
