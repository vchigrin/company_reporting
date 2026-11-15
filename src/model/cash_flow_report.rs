use super::Money;
use std::ops;

#[derive(Debug, PartialEq, Clone)]
pub struct OperationalInFlow {
    sales_income: Money,
    percent_income: Money,
    other_income: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OperationalOutFlow {
    supplier_expenses: Money,
    salaries_expenses: Money,
    percent_expenses: Money,
    tax_expenses: Money,
    other_expenses: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OperationalCashFlow {
    in_flow: OperationalInFlow,
    out_flow: OperationalOutFlow,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvestmentInFlow {
    capital_sales: Money,
    other_income: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvestmentOutFlow {
    capital_expenses: Money,
    other_expenses: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvestmentCashFlow {
    in_flow: InvestmentInFlow,
    out_flow: InvestmentOutFlow,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FinancialInFlow {
    raising_loans: Money,
    bonds_issuing: Money,
    shares_issuing: Money,
    other_income: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FinancialOutFlow {
    returning_loans: Money,
    bonds_redemption: Money,
    shares_buyback: Money,
    dividents: Money,
    other_expenses: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FinancialCashFlow {
    in_flow: FinancialInFlow,
    out_flow: FinancialOutFlow,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CashFlowReport {
    operational_flow: OperationalCashFlow,
    investment_flow: InvestmentCashFlow,
    financial_flow: FinancialCashFlow,
}

impl OperationalInFlow {
    pub fn new(sales_income: Money, percent_income: Money, other_income: Money) -> Self {
        Self {
            sales_income,
            percent_income,
            other_income,
        }
    }

    pub fn sales_income(&self) -> Money {
        self.sales_income
    }

    pub fn percent_income(&self) -> Money {
        self.percent_income
    }

    pub fn other_income(&self) -> Money {
        self.other_income
    }

    pub fn net_result(&self) -> Money {
        self.sales_income + self.percent_income + self.other_income
    }
}

impl OperationalOutFlow {
    pub fn new(
        supplier_expenses: Money,
        salaries_expenses: Money,
        percent_expenses: Money,
        tax_expenses: Money,
        other_expenses: Money,
    ) -> Self {
        Self {
            supplier_expenses,
            salaries_expenses,
            percent_expenses,
            tax_expenses,
            other_expenses,
        }
    }

    pub fn supplier_expenses(&self) -> Money {
        self.supplier_expenses
    }

    pub fn salaries_expenses(&self) -> Money {
        self.salaries_expenses
    }

    pub fn percent_expenses(&self) -> Money {
        self.percent_expenses
    }

    pub fn tax_expenses(&self) -> Money {
        self.tax_expenses
    }

    pub fn other_expenses(&self) -> Money {
        self.other_expenses
    }

    pub fn net_result(&self) -> Money {
        self.supplier_expenses
            + self.salaries_expenses
            + self.percent_expenses
            + self.tax_expenses
            + self.other_expenses
    }
}

impl OperationalCashFlow {
    pub fn new(in_flow: OperationalInFlow, out_flow: OperationalOutFlow) -> Self {
        Self { in_flow, out_flow }
    }

    pub fn in_flow(&self) -> &OperationalInFlow {
        &self.in_flow
    }

    pub fn out_flow(&self) -> &OperationalOutFlow {
        &self.out_flow
    }

    pub fn net_result(&self) -> Money {
        self.in_flow.net_result() + self.out_flow.net_result()
    }
}

impl InvestmentInFlow {
    pub fn new(capital_sales: Money, other_income: Money) -> Self {
        Self {
            capital_sales,
            other_income,
        }
    }

    pub fn capital_sales(&self) -> Money {
        self.capital_sales
    }

    pub fn other_income(&self) -> Money {
        self.other_income
    }

    pub fn net_result(&self) -> Money {
        self.capital_sales + self.other_income
    }
}

impl InvestmentOutFlow {
    pub fn new(capital_expenses: Money, other_expenses: Money) -> Self {
        Self {
            capital_expenses,
            other_expenses,
        }
    }

    pub fn capital_expenses(&self) -> Money {
        self.capital_expenses
    }

    pub fn other_expenses(&self) -> Money {
        self.other_expenses
    }

    pub fn net_result(&self) -> Money {
        self.capital_expenses + self.other_expenses
    }
}

impl InvestmentCashFlow {
    pub fn new(in_flow: InvestmentInFlow, out_flow: InvestmentOutFlow) -> Self {
        Self { in_flow, out_flow }
    }

    pub fn in_flow(&self) -> &InvestmentInFlow {
        &self.in_flow
    }

    pub fn out_flow(&self) -> &InvestmentOutFlow {
        &self.out_flow
    }

    pub fn net_result(&self) -> Money {
        self.in_flow.net_result() + self.out_flow.net_result()
    }
}

impl FinancialInFlow {
    pub fn new(
        raising_loans: Money,
        bonds_issuing: Money,
        shares_issuing: Money,
        other_income: Money,
    ) -> Self {
        Self {
            raising_loans,
            bonds_issuing,
            shares_issuing,
            other_income,
        }
    }

    pub fn raising_loans(&self) -> Money {
        self.raising_loans
    }

    pub fn bonds_issuing(&self) -> Money {
        self.bonds_issuing
    }

    pub fn shares_issuing(&self) -> Money {
        self.shares_issuing
    }

    pub fn other_income(&self) -> Money {
        self.other_income
    }

    pub fn net_result(&self) -> Money {
        self.raising_loans + self.bonds_issuing + self.shares_issuing + self.other_income
    }
}

impl FinancialOutFlow {
    pub fn new(
        returning_loans: Money,
        bonds_redemption: Money,
        shares_buyback: Money,
        dividents: Money,
        other_expenses: Money,
    ) -> Self {
        Self {
            returning_loans,
            bonds_redemption,
            shares_buyback,
            dividents,
            other_expenses,
        }
    }

    pub fn returning_loans(&self) -> Money {
        self.returning_loans
    }

    pub fn bonds_redemption(&self) -> Money {
        self.bonds_redemption
    }

    pub fn shares_buyback(&self) -> Money {
        self.shares_buyback
    }

    pub fn dividents(&self) -> Money {
        self.dividents
    }

    pub fn other_expenses(&self) -> Money {
        self.other_expenses
    }

    pub fn net_result(&self) -> Money {
        self.returning_loans
            + self.bonds_redemption
            + self.shares_buyback
            + self.dividents
            + self.other_expenses
    }
}

impl FinancialCashFlow {
    pub fn new(in_flow: FinancialInFlow, out_flow: FinancialOutFlow) -> Self {
        Self { in_flow, out_flow }
    }

    pub fn in_flow(&self) -> &FinancialInFlow {
        &self.in_flow
    }

    pub fn out_flow(&self) -> &FinancialOutFlow {
        &self.out_flow
    }

    pub fn net_result(&self) -> Money {
        self.in_flow.net_result() + self.out_flow.net_result()
    }
}

impl CashFlowReport {
    pub fn new(
        operational_flow: OperationalCashFlow,
        investment_flow: InvestmentCashFlow,
        financial_flow: FinancialCashFlow,
    ) -> Self {
        Self {
            operational_flow,
            investment_flow,
            financial_flow,
        }
    }

    pub fn operational_flow(&self) -> &OperationalCashFlow {
        &self.operational_flow
    }

    pub fn investment_flow(&self) -> &InvestmentCashFlow {
        &self.investment_flow
    }

    pub fn financial_flow(&self) -> &FinancialCashFlow {
        &self.financial_flow
    }

    pub fn net_result(&self) -> Money {
        self.operational_flow.net_result()
            + self.investment_flow.net_result()
            + self.financial_flow.net_result()
    }
}

impl ops::Sub for OperationalInFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            sales_income: self.sales_income - rhs.sales_income,
            percent_income: self.percent_income - rhs.percent_income,
            other_income: self.other_income - rhs.other_income,
        }
    }
}

impl ops::Sub for OperationalOutFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            supplier_expenses: self.supplier_expenses - rhs.supplier_expenses,
            salaries_expenses: self.salaries_expenses - rhs.salaries_expenses,
            percent_expenses: self.percent_expenses - rhs.percent_expenses,
            tax_expenses: self.tax_expenses - rhs.tax_expenses,
            other_expenses: self.other_expenses - rhs.other_expenses,
        }
    }
}

impl ops::Sub for OperationalCashFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            in_flow: self.in_flow - rhs.in_flow,
            out_flow: self.out_flow - rhs.out_flow,
        }
    }
}

impl ops::Sub for InvestmentInFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            capital_sales: self.capital_sales - rhs.capital_sales,
            other_income: self.other_income - rhs.other_income,
        }
    }
}

impl ops::Sub for InvestmentOutFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            capital_expenses: self.capital_expenses - rhs.capital_expenses,
            other_expenses: self.other_expenses - rhs.other_expenses,
        }
    }
}

impl ops::Sub for InvestmentCashFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            in_flow: self.in_flow - rhs.in_flow,
            out_flow: self.out_flow - rhs.out_flow,
        }
    }
}

impl ops::Sub for FinancialInFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            raising_loans: self.raising_loans - rhs.raising_loans,
            bonds_issuing: self.bonds_issuing - rhs.bonds_issuing,
            shares_issuing: self.shares_issuing - rhs.shares_issuing,
            other_income: self.other_income - rhs.other_income,
        }
    }
}

impl ops::Sub for FinancialOutFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            returning_loans: self.returning_loans - rhs.returning_loans,
            bonds_redemption: self.bonds_redemption - rhs.bonds_redemption,
            shares_buyback: self.shares_buyback - rhs.shares_buyback,
            dividents: self.dividents - rhs.dividents,
            other_expenses: self.other_expenses - rhs.other_expenses,
        }
    }
}

impl ops::Sub for FinancialCashFlow {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            in_flow: self.in_flow - rhs.in_flow,
            out_flow: self.out_flow - rhs.out_flow,
        }
    }
}

impl ops::Sub for CashFlowReport {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            operational_flow: self.operational_flow - rhs.operational_flow,
            investment_flow: self.investment_flow - rhs.investment_flow,
            financial_flow: self.financial_flow - rhs.financial_flow,
        }
    }
}
