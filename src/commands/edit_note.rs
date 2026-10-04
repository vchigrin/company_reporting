use clap::{ArgGroup, Args};
use eyre::{Result, eyre};

use crate::external_editor_helper::launch_editor;
use crate::storage;

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["company_inn", "company_name"])))]
pub struct EditNoteArgs {
    #[arg(long)]
    company_inn: Option<String>,
    #[arg(long)]
    company_name: Option<String>,
    #[arg(long)]
    note_id: i64,
}

pub fn process_edit_note(args: &EditNoteArgs, db: &mut storage::Storage) -> Result<()> {
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

    let editor =
        std::env::var("EDITOR").map_err(|_| eyre!("EDITOR environment variable is not set"))?;

    let temp_file = tempfile::NamedTempFile::new()?;
    // Pre-fill the temp file with the current text so the editor shows it.
    std::fs::write(temp_file.path(), &company.notes[index].text)?;
    let status = launch_editor(&editor, temp_file.path())?;
    if !status.success() {
        return Err(eyre!("Editor exited with status {}", status));
    }
    let edited_text = std::fs::read_to_string(temp_file.path())?.trim().to_owned();
    if edited_text.is_empty() {
        return Err(eyre!("Note text is empty, nothing saved"));
    }

    company.notes[index].text = edited_text;
    db.save_company(&company)?;
    let note = &company.notes[index];
    println!("Edited note {} from {}: {}", note.id, note.date, note.text);
    Ok(())
}
