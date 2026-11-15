use super::{RowDescriptor, ValueGetter};
use crate::model::{Money, Report};

fn operational_net_result(report: &Report) -> Money {
    report.cash_flow.operational_flow().net_result()
}

fn investment_net_result(report: &Report) -> Money {
    report.cash_flow.investment_flow().net_result()
}

fn financial_net_result(report: &Report) -> Money {
    report.cash_flow.financial_flow().net_result()
}

fn net_result(report: &Report) -> Money {
    report.cash_flow.net_result()
}

fn sales_income(report: &Report) -> Money {
    report.cash_flow.operational_flow().in_flow().sales_income()
}

fn percent_income(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .in_flow()
        .percent_income()
}

fn operational_other_income(report: &Report) -> Money {
    report.cash_flow.operational_flow().in_flow().other_income()
}

fn supplier_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .out_flow()
        .supplier_expenses()
}

fn salaries_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .out_flow()
        .salaries_expenses()
}

fn percent_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .out_flow()
        .percent_expenses()
}

fn tax_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .out_flow()
        .tax_expenses()
}

fn operational_other_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .operational_flow()
        .out_flow()
        .other_expenses()
}

fn capital_sales(report: &Report) -> Money {
    report.cash_flow.investment_flow().in_flow().capital_sales()
}

fn investment_other_income(report: &Report) -> Money {
    report.cash_flow.investment_flow().in_flow().other_income()
}

fn capital_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .investment_flow()
        .out_flow()
        .capital_expenses()
}

fn investment_other_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .investment_flow()
        .out_flow()
        .other_expenses()
}

fn raising_loans(report: &Report) -> Money {
    report.cash_flow.financial_flow().in_flow().raising_loans()
}

fn bonds_issuing(report: &Report) -> Money {
    report.cash_flow.financial_flow().in_flow().bonds_issuing()
}

fn shares_issuing(report: &Report) -> Money {
    report.cash_flow.financial_flow().in_flow().shares_issuing()
}

fn financial_other_income(report: &Report) -> Money {
    report.cash_flow.financial_flow().in_flow().other_income()
}

fn returning_loans(report: &Report) -> Money {
    report
        .cash_flow
        .financial_flow()
        .out_flow()
        .returning_loans()
}

fn bonds_redemption(report: &Report) -> Money {
    report
        .cash_flow
        .financial_flow()
        .out_flow()
        .bonds_redemption()
}

fn shares_buyback(report: &Report) -> Money {
    report
        .cash_flow
        .financial_flow()
        .out_flow()
        .shares_buyback()
}

fn dividents(report: &Report) -> Money {
    report.cash_flow.financial_flow().out_flow().dividents()
}

fn financial_other_expenses(report: &Report) -> Money {
    report
        .cash_flow
        .financial_flow()
        .out_flow()
        .other_expenses()
}

pub const ROWS: [RowDescriptor; 25] = [
    RowDescriptor {
        title: "Operational net result",
        value_getter: ValueGetter::Money(operational_net_result),
        level: 0,
    },
    RowDescriptor {
        title: "Sales income",
        value_getter: ValueGetter::Money(sales_income),
        level: 2,
    },
    RowDescriptor {
        title: "Percent income",
        value_getter: ValueGetter::Money(percent_income),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        value_getter: ValueGetter::Money(operational_other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Supplier expenses",
        value_getter: ValueGetter::Money(supplier_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Salaries expenses",
        value_getter: ValueGetter::Money(salaries_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Percent expenses",
        value_getter: ValueGetter::Money(percent_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Tax expenses",
        value_getter: ValueGetter::Money(tax_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        value_getter: ValueGetter::Money(operational_other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Investment net result",
        value_getter: ValueGetter::Money(investment_net_result),
        level: 0,
    },
    RowDescriptor {
        title: "Capital sales",
        value_getter: ValueGetter::Money(capital_sales),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        value_getter: ValueGetter::Money(investment_other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Capital expenses",
        value_getter: ValueGetter::Money(capital_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        value_getter: ValueGetter::Money(investment_other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Financial net result",
        value_getter: ValueGetter::Money(financial_net_result),
        level: 0,
    },
    RowDescriptor {
        title: "Raising loans",
        value_getter: ValueGetter::Money(raising_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Bonds issuing",
        value_getter: ValueGetter::Money(bonds_issuing),
        level: 2,
    },
    RowDescriptor {
        title: "Shares issuing",
        value_getter: ValueGetter::Money(shares_issuing),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        value_getter: ValueGetter::Money(financial_other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Returning loans",
        value_getter: ValueGetter::Money(returning_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Bonds redemption",
        value_getter: ValueGetter::Money(bonds_redemption),
        level: 2,
    },
    RowDescriptor {
        title: "Shares buyback",
        value_getter: ValueGetter::Money(shares_buyback),
        level: 2,
    },
    RowDescriptor {
        title: "Dividents",
        value_getter: ValueGetter::Money(dividents),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        value_getter: ValueGetter::Money(financial_other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Net cash flow result",
        value_getter: ValueGetter::Money(net_result),
        level: 0,
    },
];
