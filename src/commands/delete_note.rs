use clap::{ArgGroup, Args};
use eyre::{Result, eyre};

use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct DeleteNoteArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long)]
    note_id: i64,
}

pub fn process_delete_note(args: &DeleteNoteArgs, db: &mut storage::Storage) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };
    let Some(index) = company
        .notes
        .iter()
        .position(|note| note.id == args.note_id)
    else {
        return Err(eyre!(
            "Note with id {} not found for company {}",
            args.note_id,
            company.name
        ));
    };
    let removed = company.notes.remove(index);
    db.save_company(&company)?;
    println!(
        "Removed note {} from {}: {}",
        removed.id, removed.date, removed.text
    );
    Ok(())
}
