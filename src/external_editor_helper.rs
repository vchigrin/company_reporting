use eyre::Result;

pub fn launch_editor(editor: &str, path: &std::path::Path) -> Result<std::process::ExitStatus> {
    Ok(std::process::Command::new("sh")
        .arg("-c")
        .arg(r#"exec "$EDITOR" "$1""#)
        .arg("edit-note")
        .arg(path)
        .env("EDITOR", editor)
        .status()?)
}
