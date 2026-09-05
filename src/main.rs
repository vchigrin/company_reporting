use clap::{Args, Parser, Subcommand};
use eyre::Result;
use std::path;

mod model;
mod report_parser;

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

fn process_parse_report(args: &ParseReportArgs) -> Result<()> {
    let page_lines = report_parser::get_page_lines(&args.report_path, args.page_number)?;
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        model::ReportType::Balance => {
            let report = if args.interactive {
                parser.parse_balance_report_interactive(&page_lines, args.money_multiplier)?
            } else {
                parser.parse_balance_report_batch(&page_lines, args.money_multiplier)?
            };
            println!("Parsed balance {:?}", report);
            // TODO: save to DB.
        }
        model::ReportType::Income => {
            let report = if args.interactive {
                parser.parse_income_report_interactive(&page_lines, args.money_multiplier)?
            } else {
                parser.parse_income_report_batch(&page_lines, args.money_multiplier)?
            };
            println!("Parsed income {:?}", report);
        }
    }
    Ok(())
}

fn process_command(command: &Command) -> Result<()> {
    match command {
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
