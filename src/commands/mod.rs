mod edit_report;
mod parse_report;
mod update_reports_dict;

pub use edit_report::EditReportArgs;
pub use edit_report::process_edit_report;
pub use parse_report::ParseReportArgs;
pub use parse_report::process_parse_report;
pub use update_reports_dict::{UpdateReportsDictArgs, process_update_reports_dict};
