use super::money::Money;
use eyre::{Result, eyre};

// Money values in Income report is positive for "income" values and
// negative for "expenses" values.
//
#[derive(Debug, PartialEq)]
pub struct GrossProfitSegment {
    // Выручка от реализации
    sales_revenue: Money,
    // Ceбестоимость продаж
    cost_of_sales: Money,
}

#[derive(Debug, PartialEq)]
pub struct OperationalSegment {
    // Коммерческие расходы
    commercial_expenses: Money,
    // Управленческие расходы
    management_expenses: Money,
    // Прочие доходы
    other_income: Money,
    // Прочие расходы
    other_expenses: Money,
}

impl OperationalSegment {
    pub fn new(
        commercial_expenses: Money,
        management_expenses: Money,
        other_income: Money,
        other_expenses: Money,
    ) -> Self {
        Self {
            commercial_expenses,
            management_expenses,
            other_income,
            other_expenses,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FinancialSegment {
    // Финансовые доходы
    financial_income: Money,
    // Финансовые расходы
    financial_expenses: Money,
}

impl FinancialSegment {
    pub fn new(financial_income: Money, financial_expenses: Money) -> Self {
        Self {
            financial_income,
            financial_expenses,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IncomeReport {
    gross_profit_segment: GrossProfitSegment,
    operational_segment: OperationalSegment,
    financial_segment: FinancialSegment,
    // Налог на прибыль
    profit_tax: Money,
}

impl GrossProfitSegment {
    pub fn new(sales_revenue: Money, cost_of_sales: Money) -> Result<Self> {
        if cost_of_sales > Money::zero() {
            return Err(eyre!("Cost of sales should be negative"));
        }
        Ok(Self {
            sales_revenue,
            cost_of_sales,
        })
    }
    pub fn gross_profit(&self) -> Money {
        // cost_of_sales should be negative
        self.sales_revenue + self.cost_of_sales
    }
}

impl IncomeReport {
    pub fn new(
        gross_profit_segment: GrossProfitSegment,
        operational_segment: OperationalSegment,
        financial_segment: FinancialSegment,
        profit_tax: Money,
    ) -> Self {
        Self {
            gross_profit_segment,
            operational_segment,
            financial_segment,
            profit_tax,
        }
    }

    pub fn sales_revenue(&self) -> Money {
        self.gross_profit_segment.sales_revenue
    }

    pub fn cost_of_sales(&self) -> Money {
        self.gross_profit_segment.cost_of_sales
    }

    pub fn commercial_expenses(&self) -> Money {
        self.operational_segment.commercial_expenses
    }

    pub fn management_expenses(&self) -> Money {
        self.operational_segment.management_expenses
    }

    pub fn financial_income(&self) -> Money {
        self.financial_segment.financial_income
    }

    pub fn financial_expenses(&self) -> Money {
        self.financial_segment.financial_expenses
    }

    pub fn other_income(&self) -> Money {
        self.operational_segment.other_income
    }

    pub fn other_expenses(&self) -> Money {
        self.operational_segment.other_expenses
    }

    pub fn profit_tax(&self) -> Money {
        self.profit_tax
    }

    pub fn gross_profit(&self) -> Money {
        self.gross_profit_segment.gross_profit()
    }

    pub fn operational_profit(&self) -> Money {
        self.gross_profit()
            + self.commercial_expenses()
            + self.management_expenses()
            + self.other_income()
            + self.other_expenses()
    }

    pub fn profit_before_tax(&self) -> Money {
        self.operational_profit() + self.financial_income() + self.financial_expenses()
    }

    pub fn net_profit(&self) -> Money {
        self.profit_before_tax() + self.profit_tax
    }
}
