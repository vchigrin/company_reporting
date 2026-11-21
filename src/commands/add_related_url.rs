use clap::{ArgGroup, Args};
use eyre::Result;

use crate::model;
use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct AddRelatedUrlArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long)]
    url: String,
    #[arg(long)]
    url_kind: model::UrlKind,
}

pub fn process_add_related_url(args: &AddRelatedUrlArgs, db: &mut storage::Storage) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let related_url = model::RelatedUrl {
        url: args.url.clone(),
        url_kind: args.url_kind,
    };
    company.related_urls.push(related_url);
    db.save_company(&company)?;
    println!(
        "Saved related URL {} ({:?}) for company {}",
        args.url, args.url_kind, company.name
    );
    Ok(())
}
