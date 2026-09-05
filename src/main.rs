use clap::{Args, Parser, Subcommand};
use eyre::Result;
use std::path;

mod model;
mod report_parser;

#[derive(Debug, Args)]
struct UpdateDbArgs {
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
}

#[derive(Debug, Subcommand)]
enum Command {
    UpdateDb(UpdateDbArgs),
}

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Tool for parsing and evaluation company financial reports"
)]
struct CliParams {
    #[command(subcommand)]
    command: Command,
    #[arg(long("db"))]
    db_path: path::PathBuf,
}

fn process_update_db(db_path: &path::Path, args: &UpdateDbArgs) -> Result<()> {
    let page_lines = report_parser::get_page_lines(&args.report_path, args.page_number)?;
    let parser = report_parser::ReportParser::new();
    match args.report_type {
        model::ReportType::Balance => {
            let report = if args.interactive {
                parser.parse_balance_report_interactive(&page_lines)?
            } else {
                parser.parse_balance_report_batch(&page_lines)?
            };
            println!("Parsed balance {:?}", report);
            // TODO: save to DB.
        }
        model::ReportType::Income => {
            let report = if args.interactive {
                parser.parse_income_report_interactive(&page_lines)?
            } else {
                parser.parse_income_report_batch(&page_lines)?
            };
            println!("Parsed income {:?}", report);
        }
    }
    Ok(())
}

fn process_command(db_path: &path::Path, command: &Command) -> Result<()> {
    match command {
        Command::UpdateDb(update_db_args) => process_update_db(db_path, update_db_args),
    }
}

fn do_main() -> Result<()> {
    let params = CliParams::parse();
    println!("Running with params {:?}", params);
    process_command(&params.db_path, &params.command)?;
    Ok(())
}

fn main() {
    simple_logging::log_to_file("company_reporting.log", log::LevelFilter::Debug).unwrap();
    if let Err(e) = do_main() {
        println!("Error {}", e);
        std::process::exit(1);
    }
}
