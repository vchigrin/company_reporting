mod add_note;
mod add_ra_conclusion;
mod delete_note;
mod detailed_company_report;
mod edit_note;
mod edit_report;
mod parse_report;
mod update_reports_dict;

pub use add_note::{AddNoteArgs, process_add_note};
pub use add_ra_conclusion::{AddRAConclusionArgs, process_add_ra_conclusion};
pub use delete_note::{DeleteNoteArgs, process_delete_note};
pub use detailed_company_report::{DetailedReportArgs, process_detailed_company_report};
pub use edit_note::{EditNoteArgs, process_edit_note};
pub use edit_report::EditReportArgs;
pub use edit_report::process_edit_report;
pub use parse_report::ParseReportArgs;
pub use parse_report::process_parse_report;
pub use update_reports_dict::{UpdateReportsDictArgs, process_update_reports_dict};

use crate::model;
use strum::VariantArray;

impl clap::ValueEnum for model::ReportType {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let val: &'static str = self.into();
        Some(clap::builder::PossibleValue::new(val))
    }

    fn value_variants<'a>() -> &'a [Self] {
        Self::VARIANTS
    }
}

impl clap::ValueEnum for model::MoneyMultiplier {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let val: &'static str = self.into();
        Some(clap::builder::PossibleValue::new(val))
    }

    fn value_variants<'a>() -> &'a [Self] {
        Self::VARIANTS
    }
}
