use clap::{ArgGroup, Args};
use eyre::{Result, eyre};

use crate::external_editor_helper::launch_editor;
use crate::model;
use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct AddNoteArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
}

pub fn process_add_note(args: &AddNoteArgs, db: &mut storage::Storage) -> Result<()> {
    let mut company = match (&args.company_inn, &args.company_name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };

    let editor =
        std::env::var("EDITOR").map_err(|_| eyre!("EDITOR environment variable is not set"))?;

    let temp_file = tempfile::NamedTempFile::new()?;
    let status = launch_editor(&editor, temp_file.path())?;
    if !status.success() {
        return Err(eyre!("Editor exited with status {}", status));
    }
    let text = std::fs::read_to_string(temp_file.path())?.trim().to_owned();
    if text.is_empty() {
        return Err(eyre!("Note text is empty, nothing saved"));
    }

    let date = time::OffsetDateTime::now_local()?.date();
    let note = model::Note {
        id: 0,
        date,
        text: text.clone(),
    };
    company.notes.push(note);
    // Keep the invariant that notes are sorted by date.
    company.notes.sort_by_key(|note| note.date);
    db.save_company(&company)?;
    println!("Saved note from {}: {}", date, text);
    Ok(())
}
