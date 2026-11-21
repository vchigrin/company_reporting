use clap::{ArgGroup, Args};
use eyre::Result;

use crate::model;
use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct OpenRelatedUrlsArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long)]
    url_kind: Option<model::UrlKind>,
}

pub fn process_open_related_urls(
    args: &OpenRelatedUrlsArgs,
    db: &mut storage::Storage,
) -> Result<()> {
    let company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let urls: Vec<_> = company
        .related_urls
        .iter()
        .filter(|related_url| match args.url_kind {
            Some(url_kind) => related_url.url_kind == url_kind,
            None => true,
        })
        .collect();
    if urls.is_empty() {
        println!("No related URLs found for company {}", company.name);
        return Ok(());
    }
    for related_url in urls {
        webbrowser::open(&related_url.url)?;
        println!("Opened {}", related_url.url);
    }
    Ok(())
}
