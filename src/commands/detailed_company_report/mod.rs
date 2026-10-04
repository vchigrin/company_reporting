use crate::model::{Money, Report};
use std::fmt;

mod balance_report;
mod command;
mod income_report;
mod metrics_report;

pub use command::DetailedReportArgs;
pub use command::process_detailed_company_report;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum Metric {
    Money(Money),
    Ratio(f64),
}

impl Metric {
    fn percent_increase(prev: Metric, cur: Metric) -> f64 {
        match prev {
            Metric::Money(prev_money) => {
                let Metric::Money(cur_money) = cur else {
                    panic!("Heteroheneus percent attempt");
                };
                if prev_money != Money::zero() {
                    ((cur_money - prev_money).in_roubles() as f64 * 100.)
                        / (prev_money.in_roubles() as f64)
                } else {
                    f64::INFINITY
                }
            }
            Metric::Ratio(prev_ratio) => {
                let Metric::Ratio(cur_ratio) = cur else {
                    panic!("Heteroheneus percent attempt");
                };
                if prev_ratio != 0. {
                    ((cur_ratio - prev_ratio) * 100.) / (prev_ratio)
                } else {
                    f64::INFINITY
                }
            }
        }
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Metric::Money(m) => m.fmt(f),
            Metric::Ratio(r) => r.fmt(f),
        }
    }
}

type MoneyGetter = fn(&Report) -> Money;

struct MetricInput {
    // Last twelve month reports, sorted (most recent period last).
    // Guaranted not empty and contain all required data for last year.
    // If no data for last twelve month present, then None.
    reports_ltm: Option<Vec<Report>>,
    last_report: Report,
}

type MetricGetter = fn(&MetricInput) -> Option<Metric>;

enum ValueGetter {
    None,
    Money(MoneyGetter),
    Metric(MetricGetter),
}

struct RowDescriptor {
    title: &'static str,
    value_getter: ValueGetter,
    level: i32,
}
