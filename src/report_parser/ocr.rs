use eyre::{Result, eyre};
use std::path;
use std::path::Path;
use std::process;

fn make_png(pdf_file_path: &Path, page_number: i32, dst_dir: &path::Path) -> Result<path::PathBuf> {
    const TEMP_PNG_PREFIX: &str = "page";
    let number_str = page_number.to_string();
    let dst_prefix = dst_dir.join(TEMP_PNG_PREFIX);
    let mut cmd = process::Command::new("pdftoppm");
    cmd.args(["-f", &number_str])
        .args(["-l", &number_str])
        .args(["-r", "300"]) // Increase DPI for better tesseract recognizing.
        .args(["-singlefile", "-png"])
        .arg(pdf_file_path)
        .arg(&dst_prefix);
    let make_png_status = cmd.status()?;
    if !make_png_status.success() {
        return Err(eyre!(
            "Failed generage PNG file; Command {:?} Exited with {:?}",
            cmd,
            make_png_status
        ));
    }
    Ok(dst_prefix.with_extension("png"))
}

fn perform_ocr(png_file_path: &Path) -> Result<Vec<String>> {
    let mut cmd = process::Command::new("tesseract");
    cmd.args(["-l", "rus"])
        .args(["-c", "preserve_interword_spaces=1"])
        .args(["--psm", "6"])
        .arg(png_file_path)
        .arg("stdout");
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "Failed generage PNG file; Command {:?} Exited with {:?}",
            cmd,
            output.status
        ));
    }
    let out = String::from_utf8(output.stdout)?;
    let lines: Vec<String> = out.split('\n').map(|l| l.to_owned()).collect();
    Ok(lines)
}

pub fn get_page_lines(pdf_file_path: &Path, page_number: i32) -> Result<Vec<String>> {
    let dst_dir = tempfile::TempDir::new()?;
    let png_file_path = make_png(pdf_file_path, page_number, dst_dir.path())?;
    perform_ocr(&png_file_path)
}
