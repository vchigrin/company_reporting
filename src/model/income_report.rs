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
    ) -> Result<Self> {
        if commercial_expenses > Money::zero() {
            return Err(eyre!("commercial_expenses must be non-positive"));
        }
        if management_expenses > Money::zero() {
            return Err(eyre!("management_expenses must be non-positive"));
        }
        if other_expenses > Money::zero() {
            return Err(eyre!("other_expenses must be non-positive"));
        }
        if other_income < Money::zero() {
            return Err(eyre!("other_income must be non-negative"));
        }
        Ok(Self {
            commercial_expenses,
            management_expenses,
            other_income,
            other_expenses,
        })
    }

    pub fn operational_expenses(&self) -> Money {
        self.commercial_expenses
            + self.management_expenses
            + self.other_income
            + self.other_expenses
    }

    pub fn commercial_expenses(&self) -> Money {
        self.commercial_expenses
    }

    pub fn management_expenses(&self) -> Money {
        self.management_expenses
    }

    pub fn other_income(&self) -> Money {
        self.other_income
    }

    pub fn other_expenses(&self) -> Money {
        self.other_expenses
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
    pub fn new(financial_income: Money, financial_expenses: Money) -> Result<Self> {
        if financial_expenses > Money::zero() {
            return Err(eyre!("financial_expenses must be non-positive"));
        }
        if financial_income < Money::zero() {
            return Err(eyre!("financial_income must be non-negative"));
        }
        Ok(Self {
            financial_income,
            financial_expenses,
        })
    }

    pub fn net_financial_expenses(&self) -> Money {
        self.financial_income + self.financial_expenses
    }

    pub fn financial_income(&self) -> Money {
        self.financial_income
    }

    pub fn financial_expenses(&self) -> Money {
        self.financial_expenses
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
        if sales_revenue < Money::zero() {
            return Err(eyre!("Sales revenue must be non-negative"));
        }
        if cost_of_sales > Money::zero() {
            return Err(eyre!("Cost of sales should be negative"));
        }
        Ok(Self {
            sales_revenue,
            cost_of_sales,
        })
    }
    pub fn sales_revenue(&self) -> Money {
        self.sales_revenue
    }

    pub fn cost_of_sales(&self) -> Money {
        self.cost_of_sales
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

    pub fn gross_profit_segment(&self) -> &GrossProfitSegment {
        &self.gross_profit_segment
    }

    pub fn operational_segment(&self) -> &OperationalSegment {
        &self.operational_segment
    }

    pub fn financial_segment(&self) -> &FinancialSegment {
        &self.financial_segment
    }

    pub fn profit_tax(&self) -> Money {
        self.profit_tax
    }

    pub fn gross_profit(&self) -> Money {
        self.gross_profit_segment.gross_profit()
    }

    pub fn operational_profit(&self) -> Money {
        let gross_profit = self.gross_profit();
        gross_profit + self.operational_segment.operational_expenses()
    }

    pub fn profit_before_tax(&self) -> Money {
        let operational_profit = self.operational_profit();
        operational_profit + self.financial_segment.net_financial_expenses()
    }

    pub fn net_profit(&self) -> Money {
        let profit_before_tax = self.profit_before_tax();
        profit_before_tax + self.profit_tax
    }
}
