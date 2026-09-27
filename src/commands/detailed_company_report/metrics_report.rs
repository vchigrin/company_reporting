use super::{Metric, MetricInput, RowDescriptor, ValueGetter};
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

fn ebit_ltm(metric_input: &MetricInput) -> Option<Metric> {
    if let Some(reports_ltm) = &metric_input.reports_ltm {
        let mut result = Money::zero();
        for r in reports_ltm {
            result += r.income.operational_profit();
        }
        Some(Metric::Money(result))
    } else {
        None
    }
}

fn net_debt_ebit_ltm(metric_input: &MetricInput) -> Option<Metric> {
    let ebit_ltm_metric = ebit_ltm(metric_input)?;
    let net_debt = net_debt(&metric_input.last_report);
    let Metric::Money(ebit_ltm) = ebit_ltm_metric else {
        panic!("Unexpected ebit_ltm metric type");
    };
    let result = (net_debt.in_roubles() as f64) / (ebit_ltm.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

fn icr(metric_input: &MetricInput) -> Option<Metric> {
    let last_income = &metric_input.last_report.income;
    let operational_profit = last_income.operational_profit();
    let net_financial_expenses = last_income
        .financial_segment()
        .net_financial_expenses()
        .abs();
    let result =
        (operational_profit.in_roubles() as f64) / (net_financial_expenses.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

fn liquidity_current_ratio(metric_input: &MetricInput) -> Option<Metric> {
    let last_balance = &metric_input.last_report.balance;
    let total_liabilities = last_balance.liabilities().current().total();
    let current_assets = last_balance.assets().current().total();
    let result = (current_assets.in_roubles() as f64) / (total_liabilities.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

fn liquidity_quick_ratio(metric_input: &MetricInput) -> Option<Metric> {
    let last_balance = &metric_input.last_report.balance;
    let total_liabilities = last_balance.liabilities().current().total();
    let estimated_assets = last_balance.assets().current().total()
        - last_balance.assets().current().physical_inventory()
        - last_balance.assets().current().other();
    let result = (estimated_assets.in_roubles() as f64) / (total_liabilities.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

fn liquidity_cash_ratio(metric_input: &MetricInput) -> Option<Metric> {
    let last_balance = &metric_input.last_report.balance;
    let total_liabilities = last_balance.liabilities().current().total();
    let estimated_assets = last_balance.assets().current().total()
        - last_balance.assets().current().physical_inventory()
        - last_balance.assets().current().accounts_receivable()
        - last_balance.assets().current().other();
    let result = (estimated_assets.in_roubles() as f64) / (total_liabilities.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

fn debt_to_equity_ratio(metric_input: &MetricInput) -> Option<Metric> {
    let last_balance = &metric_input.last_report.balance;
    let debt = last_balance.liabilities().total();
    let equity = last_balance.equity().total();
    let result = (debt.in_roubles() as f64) / (equity.in_roubles() as f64);
    Some(Metric::Ratio(result))
}

pub const ROWS: [RowDescriptor; 9] = [
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
    RowDescriptor {
        title: "Liquidity metrics",
        value_getter: ValueGetter::None,
        level: 0,
    },
    RowDescriptor {
        // "Red" and "Yellow" thresholds for ratio.
        // TODO: Guess we should color-encode them...
        title: "Current ratio (<1 - r, <1.5 - y)",
        value_getter: ValueGetter::Metric(liquidity_current_ratio),
        level: 2,
    },
    RowDescriptor {
        title: "Quick ratio (<0.7 - r, <1 - y)",
        value_getter: ValueGetter::Metric(liquidity_quick_ratio),
        level: 2,
    },
    RowDescriptor {
        title: "Cash ratio (<0.2 - r, <0.5 - y)",
        value_getter: ValueGetter::Metric(liquidity_cash_ratio),
        level: 2,
    },
    RowDescriptor {
        title: "Debt/Equitiy (>2 - r, >1.5 - y)",
        value_getter: ValueGetter::Metric(debt_to_equity_ratio),
        level: 0,
    },
];
