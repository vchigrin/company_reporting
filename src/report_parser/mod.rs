pub mod lines_classifier;
mod ocr;
mod text_parser;

pub use ocr::get_page_lines;
pub use text_parser::ReportParser;

use crate::model::{GenericKeys, Money};
use eyre::Report;

// Information about the change, that required to fix the problematic lines range.
// Usually describes modification of the "other" key.
#[derive(Debug)]
pub struct FixInfo<Keys: GenericKeys> {
    pub lines_range: std::ops::Range<usize>,
    pub line_key_to_modify: Keys,
    // Either positive or negative
    pub add_amount: Money,
}

#[derive(Debug)]
pub enum ParseError<Keys: GenericKeys> {
    SumMismatch { fix: FixInfo<Keys>, message: String },
    Generic(Report),
}
