mod interactive_lines_editor;
mod lines_classifier;
mod ocr;
mod text_parser;

pub use ocr::get_page_lines;
pub use text_parser::ReportParser;

use crate::model::Money;
use strum::VariantArray;

trait GenericKeys:
    PartialEq + Clone + Copy + Default + std::fmt::Debug + std::str::FromStr + VariantArray
{
}

#[derive(Debug, Clone)]
struct ParsedLineInfo<Keys: GenericKeys> {
    key: Keys,
    value: Money,
    original_line: String,
}
