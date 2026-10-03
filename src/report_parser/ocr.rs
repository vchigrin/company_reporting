use eyre::{Result, eyre};
use std::ops::RangeInclusive;
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

fn perform_ocr_non_table(img_file_path: &Path) -> Result<Vec<String>> {
    let mut cmd = process::Command::new("tesseract");
    cmd.args(["-l", "rus+eng"])
        .args(["-c", "preserve_interword_spaces=1"])
        .args(["--psm", "6"])
        .arg(img_file_path)
        .arg("stdout");
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "Failed recognize image file; Command {:?} Exited with {:?}",
            cmd,
            output.status
        ));
    }
    let out = String::from_utf8(output.stdout)?;
    let lines: Vec<String> = out.split('\n').map(|l| l.to_owned()).collect();
    Ok(lines)
}

fn perform_ocr_table(tmp_dir: &Path, png_file_path: &Path) -> Result<Vec<String>> {
    // Some crazy hack - tesseract in get.images makes
    // .tiff file with table lines removed, that can be processed by
    // non_table process algoright im second tesseract call.
    // Guess, there should be more elegant way of doing this, but for now
    // lets try this.
    let mut cmd = process::Command::new("tesseract");
    const BASE_NAME: &str = "tmp-image";
    cmd.arg(png_file_path);
    cmd.args([BASE_NAME, "get.images"]);
    cmd.current_dir(tmp_dir);
    let status = cmd.status()?;
    if !status.success() {
        return Err(eyre!(
            "Failed generage intermediate file; Command {:?} Exited with {:?}",
            cmd,
            status
        ));
    }
    let tmp_file = tmp_dir.join(format!("{}.processed.tif", BASE_NAME));
    perform_ocr_non_table(&tmp_file)
}

pub fn get_page_lines(
    pdf_file_path: &Path,
    page_numbers: RangeInclusive<i32>,
    rsbu_mode: bool,
) -> Result<Vec<String>> {
    let dst_dir = tempfile::TempDir::new()?;
    let mut result = Vec::new();
    for page_number in page_numbers {
        let png_file_path = make_png(pdf_file_path, page_number, dst_dir.path())?;
        let page_lines = if rsbu_mode {
            perform_ocr_table(dst_dir.path(), &png_file_path)?
        } else {
            perform_ocr_non_table(&png_file_path)?
        };
        result.extend(page_lines);
    }
    Ok(result)
}
