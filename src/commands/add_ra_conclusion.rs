use clap::{ArgGroup, Args};
use eyre::Result;
use time::Date;

use crate::model;
use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct AddRAConclusionArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long, value_parser = parse_date)]
    date: Date,
    #[arg(long)]
    rating_agency: model::RatingAgency,
    #[arg(long)]
    rating: model::Rating,
    #[arg(long)]
    forecast: model::RatingForecast,
}

fn parse_date(s: &str) -> Result<Date, String> {
    Date::parse(s, &time::format_description::well_known::Iso8601::DATE).map_err(|e| e.to_string())
}

pub fn process_add_ra_conclusion(
    args: &AddRAConclusionArgs,
    db: &mut storage::Storage,
) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let conclusion = model::RAConclusion {
        date: args.date,
        rating_agency: args.rating_agency,
        rating: args.rating,
        forecast: args.forecast,
    };
    company.ra_conclusions.push(conclusion);
    // Keep the invariant that conclusions are sorted by date.
    company.ra_conclusions.sort_by_key(|c| c.date);
    db.save_company(&company)?;
    println!("Saved RA conclusion {:?}", conclusion);
    Ok(())
}
