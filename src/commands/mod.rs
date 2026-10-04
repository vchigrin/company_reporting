mod add_ra_conclusion;
mod detailed_company_report;
mod edit_report;
mod parse_report;
mod update_reports_dict;

pub use add_ra_conclusion::{AddRAConclusionArgs, process_add_ra_conclusion};
pub use detailed_company_report::{DetailedReportArgs, process_detailed_company_report};
pub use edit_report::EditReportArgs;
pub use edit_report::process_edit_report;
pub use parse_report::ParseReportArgs;
pub use parse_report::process_parse_report;
pub use update_reports_dict::{UpdateReportsDictArgs, process_update_reports_dict};
