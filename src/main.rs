use clap::{Args, Parser, Subcommand, ValueEnum};
use color_print::cprintln;
use eyre::{Result, eyre};
use std::path;

mod model;

#[derive(Debug, Clone, ValueEnum)]
enum ReportType {
    Balance,
    Income,
}

#[derive(Debug, Args)]
struct UpdateDbArgs {
    #[arg(long)]
    company_name: String,
    #[arg(long, value_parser=model::Period::from_short_string)]
    period: model::Period,
    #[arg(long)]
    report_type: ReportType,
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

fn handle_command(db_path: &path::Path, command: &Command) -> Result<()> {
    Ok(())
}

fn do_main() -> Result<()> {
    let params = CliParams::parse();
    println!("Running with params {:?}", params);
    handle_command(&params.db_path, &params.command)?;
    Ok(())
}

fn main() {
    if let Err(e) = do_main() {
        println!("Error {}", e);
        std::process::exit(1);
    }
}
