use super::{RowDescriptor, ValueGetter};
use crate::model::{Money, Report};

fn gross_profit(report: &Report) -> Money {
    report.income.gross_profit()
}

fn sales_revenue(report: &Report) -> Money {
    report.income.gross_profit_segment().sales_revenue()
}

fn cost_of_sales(report: &Report) -> Money {
    report.income.gross_profit_segment().cost_of_sales()
}

fn operational_expenses(report: &Report) -> Money {
    report.income.operational_segment().operational_expenses()
}

fn commercial_expenses(report: &Report) -> Money {
    report.income.operational_segment().commercial_expenses()
}

fn management_expenses(report: &Report) -> Money {
    report.income.operational_segment().management_expenses()
}

fn other_income(report: &Report) -> Money {
    report.income.operational_segment().other_income()
}

fn other_expenses(report: &Report) -> Money {
    report.income.operational_segment().other_expenses()
}

fn operational_profit(report: &Report) -> Money {
    report.income.operational_profit()
}

fn net_financial_expenses(report: &Report) -> Money {
    report.income.financial_segment().net_financial_expenses()
}

fn financial_income(report: &Report) -> Money {
    report.income.financial_segment().financial_income()
}

fn financial_expenses(report: &Report) -> Money {
    report.income.financial_segment().financial_expenses()
}

fn profit_tax(report: &Report) -> Money {
    report.income.profit_tax()
}

fn profit_before_tax(report: &Report) -> Money {
    report.income.profit_before_tax()
}

fn net_profit(report: &Report) -> Money {
    report.income.net_profit()
}

pub const ROWS: [RowDescriptor; 15] = [
    RowDescriptor {
        title: "Gross profit",
        value_getter: ValueGetter::Money(gross_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Sales revenue",
        value_getter: ValueGetter::Money(sales_revenue),
        level: 2,
    },
    RowDescriptor {
        title: "Cost of sales",
        value_getter: ValueGetter::Money(cost_of_sales),
        level: 2,
    },
    RowDescriptor {
        title: "Operational expenses",
        value_getter: ValueGetter::Money(operational_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Comercial expenses",
        value_getter: ValueGetter::Money(commercial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Management expenses",
        value_getter: ValueGetter::Money(management_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        value_getter: ValueGetter::Money(other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        value_getter: ValueGetter::Money(other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Operational profit",
        value_getter: ValueGetter::Money(operational_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Financial net expenses",
        value_getter: ValueGetter::Money(net_financial_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Financial income",
        value_getter: ValueGetter::Money(financial_income),
        level: 2,
    },
    RowDescriptor {
        title: "Financial expenses",
        value_getter: ValueGetter::Money(financial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Profit before tax",
        value_getter: ValueGetter::Money(profit_before_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Profit tax",
        value_getter: ValueGetter::Money(profit_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Net profit",
        value_getter: ValueGetter::Money(net_profit),
        level: 0,
    },
];
