use super::{Metric, RowDescriptor, ValueGetter};
use crate::model::{Money, Report};

fn net_debt(report: &Report) -> Money {
    let current_assets = report.balance.assets().current();
    // TODO(vchigrin): May be we should move financial assets to
    // kind of options here, may be using them not always a good idea?
    let liquidity = current_assets.cash() + current_assets.financial_assets();

    let liabilities = report.balance.liabilities();
    let percent_debt = liabilities.current().loans() + liabilities.long_term().loans();
    percent_debt - liquidity
}

fn ebit_ltm(reports_ltm: &[Report]) -> Metric {
    let mut result = Money::zero();
    for r in reports_ltm {
        result += r.income.operational_profit();
    }
    Metric::Money(result)
}

fn net_debt_ebit_ltm(reports_ltm: &[Report]) -> Metric {
    // reports_ltm must not be empty.
    let net_debt = net_debt(reports_ltm.last().unwrap());
    let Metric::Money(ebit_ltm) = ebit_ltm(reports_ltm) else {
        panic!("Unexpected ebit_ltm metric type");
    };
    let result = (net_debt.in_roubles() as f64) / (ebit_ltm.in_roubles() as f64);
    Metric::Ratio(result)
}

fn icr(reports_ltm: &[Report]) -> Metric {
    // reports_ltm must not be empty.
    let last_income = &reports_ltm.last().unwrap().income;
    let operational_profit = last_income.operational_profit();
    let net_financial_expenses = last_income
        .financial_segment()
        .net_financial_expenses()
        .abs();
    let result =
        (operational_profit.in_roubles() as f64) / (net_financial_expenses.in_roubles() as f64);
    Metric::Ratio(result)
}

pub const ROWS: [RowDescriptor; 4] = [
    RowDescriptor {
        title: "Net debt",
        value_getter: ValueGetter::Money(net_debt),
        level: 0,
    },
    RowDescriptor {
        title: "EBIT LTM",
        value_getter: ValueGetter::Metric(ebit_ltm),
        level: 0,
    },
    RowDescriptor {
        title: "Net debt/EBIT LTM",
        value_getter: ValueGetter::Metric(net_debt_ebit_ltm),
        level: 0,
    },
    RowDescriptor {
        title: "ICR",
        value_getter: ValueGetter::Metric(icr),
        level: 0,
    },
];
