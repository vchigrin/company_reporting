use clap::{ArgGroup, Args, Parser, Subcommand};
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::path;

mod commands;
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
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["inn", "name"])))]
struct GetCompanyArgs {
    #[arg(long)]
    inn: Option<String>,
    #[arg(long)]
    name: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Command {
    AddCompany(AddCompanyArgs),
    GetCompany(GetCompanyArgs),
    ListCompanies,
    UpdateDictFromReports(commands::UpdateReportsDictArgs),
    EditReport(commands::EditReportArgs),
    ParseReport(commands::ParseReportArgs),
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
    if db.get_company_by_name(&args.name).is_ok() {
        return Err(eyre!("Company with name {} already present", args.name));
    }
    if db.get_company_by_inn(&args.inn).is_ok() {
        return Err(eyre!("Company with inn {} already present", args.inn));
    }
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
    let company = match (&args.inn, &args.name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
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
        Command::UpdateDictFromReports(args) => {
            let mut db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
            commands::process_update_reports_dict(args, &mut db)
        }
        Command::EditReport(args) => {
            let mut db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
            commands::process_edit_report(args, &mut db)
        }
        Command::ParseReport(args) => {
            let mut db = storage::Storage::new_with_file(path::Path::new(DB_FILE_PATH))?;
            commands::process_parse_report(args, &mut db)
        }
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
