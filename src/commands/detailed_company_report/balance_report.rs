use super::{RowDescriptor, ValueGetter};
use crate::model::{Money, Report};

fn total_assets(report: &Report) -> Money {
    report.balance.assets().total()
}

fn total_non_current_assets(report: &Report) -> Money {
    report.balance.assets().non_current().total()
}

fn total_current_assets(report: &Report) -> Money {
    report.balance.assets().current().total()
}

fn non_material_assets(report: &Report) -> Money {
    report.balance.assets().non_current().non_material_assets()
}

fn fixed_assets(report: &Report) -> Money {
    report.balance.assets().non_current().fixed_assets()
}

fn non_current_financial_assets(report: &Report) -> Money {
    report.balance.assets().non_current().financial_assets()
}

fn non_current_other(report: &Report) -> Money {
    report.balance.assets().non_current().other()
}

fn physical_inventory(report: &Report) -> Money {
    report.balance.assets().current().physical_inventory()
}

fn accounts_receivable(report: &Report) -> Money {
    report.balance.assets().current().accounts_receivable()
}

fn current_financial_assets(report: &Report) -> Money {
    report.balance.assets().current().financial_assets()
}

fn cash(report: &Report) -> Money {
    report.balance.assets().current().cash()
}

fn current_other(report: &Report) -> Money {
    report.balance.assets().current().other()
}

fn total_equity(report: &Report) -> Money {
    report.balance.equity().total()
}

fn total_liabilities(report: &Report) -> Money {
    report.balance.liabilities().total()
}

fn total_long_term_liabilities(report: &Report) -> Money {
    report.balance.liabilities().long_term().total()
}

fn total_current_liabilities(report: &Report) -> Money {
    report.balance.liabilities().current().total()
}

fn authorised_capital(report: &Report) -> Money {
    report.balance.equity().authorised_capital()
}

fn capital_surplus(report: &Report) -> Money {
    report.balance.equity().capital_surplus()
}

fn retained_earnings(report: &Report) -> Money {
    report.balance.equity().retained_earnings()
}

fn equity_other(report: &Report) -> Money {
    report.balance.equity().other()
}

fn long_term_loans(report: &Report) -> Money {
    report.balance.liabilities().long_term().loans()
}

fn long_term_accounts_payable(report: &Report) -> Money {
    report.balance.liabilities().long_term().accounts_payable()
}

fn long_term_other(report: &Report) -> Money {
    report.balance.liabilities().long_term().other()
}

fn current_loans(report: &Report) -> Money {
    report.balance.liabilities().current().loans()
}

fn current_accounts_payable(report: &Report) -> Money {
    report.balance.liabilities().current().accounts_payable()
}

fn current_liabilities_other(report: &Report) -> Money {
    report.balance.liabilities().current().other()
}

pub const ROWS: [RowDescriptor; 26] = [
    RowDescriptor {
        title: "Assets",
        value_getter: ValueGetter::Money(total_assets),
        level: 0,
    },
    RowDescriptor {
        title: "Non-current assets",
        value_getter: ValueGetter::Money(total_non_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Non-material assets",
        value_getter: ValueGetter::Money(non_material_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Fixed assets",
        value_getter: ValueGetter::Money(fixed_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (non-current)",
        value_getter: ValueGetter::Money(non_current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Other non-current assets",
        value_getter: ValueGetter::Money(non_current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Current assets",
        value_getter: ValueGetter::Money(total_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Inventory",
        value_getter: ValueGetter::Money(physical_inventory),
        level: 2,
    },
    RowDescriptor {
        title: "Accounts receivable",
        value_getter: ValueGetter::Money(accounts_receivable),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (current)",
        value_getter: ValueGetter::Money(current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Cash and equivalents",
        value_getter: ValueGetter::Money(cash),
        level: 2,
    },
    RowDescriptor {
        title: "Other current assets",
        value_getter: ValueGetter::Money(current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Equity",
        value_getter: ValueGetter::Money(total_equity),
        level: 0,
    },
    RowDescriptor {
        title: "Authorised capital",
        value_getter: ValueGetter::Money(authorised_capital),
        level: 2,
    },
    RowDescriptor {
        title: "Capital surplus",
        value_getter: ValueGetter::Money(capital_surplus),
        level: 2,
    },
    RowDescriptor {
        title: "Retained earnings",
        value_getter: ValueGetter::Money(retained_earnings),
        level: 2,
    },
    RowDescriptor {
        title: "Other equity",
        value_getter: ValueGetter::Money(equity_other),
        level: 2,
    },
    RowDescriptor {
        title: "Liabilities",
        value_getter: ValueGetter::Money(total_liabilities),
        level: 0,
    },
    RowDescriptor {
        title: "Long-term liabilities",
        value_getter: ValueGetter::Money(total_long_term_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Long-term loans",
        value_getter: ValueGetter::Money(long_term_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Long-term accounts payable",
        value_getter: ValueGetter::Money(long_term_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other long-term liabilities",
        value_getter: ValueGetter::Money(long_term_other),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term liabilities",
        value_getter: ValueGetter::Money(total_current_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Short-term loans",
        value_getter: ValueGetter::Money(current_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term accounts payable",
        value_getter: ValueGetter::Money(current_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other current liabilities",
        value_getter: ValueGetter::Money(current_liabilities_other),
        level: 2,
    },
];
