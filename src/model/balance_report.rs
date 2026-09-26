use super::money::Money;
use eyre::{Result, eyre};

// Необоротные активы.
#[derive(Debug, PartialEq, Clone)]
pub struct NonCurrentAssets {
    // Основные средства
    fixed_assets: Money,
    // Нематериальные активы
    non_material_assets: Money,
    // Финансовые вложения
    financial_assets: Money,
    // Прочее
    other: Money,
}

// Оборотные активы
#[derive(Debug, PartialEq, Clone)]
pub struct CurrentAssets {
    // Запасы
    physical_inventory: Money,
    // Дебиторская задолженность
    accounts_receivable: Money,
    // Денежные средства и их эквиваленты
    cash: Money,
    // Краткосрочные финансовые вложения
    financial_assets: Money,
    // Прочее
    other: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assets {
    current: CurrentAssets,
    non_current: NonCurrentAssets,
}

// Капитал
#[derive(Debug, PartialEq, Clone)]
pub struct Equity {
    // Уставной капитал
    authorised_capital: Money,
    // Добавочный капитал
    capital_surplus: Money,
    // Нераспределённая прибыль (непокрытый убыток)
    retained_earnings: Money,
    other: Money,
}

// Долгосрочные обязательства
#[derive(Debug, PartialEq, Clone)]
pub struct LongTermLiabilities {
    // Кредиты и займы
    loans: Money,
    // Кредиторская задолженность
    accounts_payable: Money,
    // Прочее
    other: Money,
}

// Краткосрочные обязательства
#[derive(Debug, PartialEq, Clone)]
pub struct CurrentLiabilities {
    // Кредиты и займы
    loans: Money,
    // Кредиторская задолженность
    accounts_payable: Money,
    // Прочее
    other: Money,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Liabilities {
    long_term: LongTermLiabilities,
    current: CurrentLiabilities,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BalanceReport {
    assets: Assets,
    equity: Equity,
    liabilities: Liabilities,
}

impl NonCurrentAssets {
    pub fn new(
        // Основные средства
        fixed_assets: Money,
        // Нематериальные активы
        non_material_assets: Money,
        // Финансовые вложения
        financial_assets: Money,
        // Прочее
        other: Money,
    ) -> Self {
        Self {
            fixed_assets,
            non_material_assets,
            financial_assets,
            other,
        }
    }

    pub fn fixed_assets(&self) -> Money {
        self.fixed_assets
    }

    pub fn non_material_assets(&self) -> Money {
        self.non_material_assets
    }

    pub fn financial_assets(&self) -> Money {
        self.financial_assets
    }

    pub fn other(&self) -> Money {
        self.other
    }

    pub fn total(&self) -> Money {
        self.fixed_assets + self.non_material_assets + self.financial_assets + self.other
    }
}

impl CurrentAssets {
    pub fn new(
        physical_inventory: Money,
        accounts_receivable: Money,
        cash: Money,
        financial_assets: Money,
        other: Money,
    ) -> Self {
        Self {
            physical_inventory,
            accounts_receivable,
            cash,
            financial_assets,
            other,
        }
    }

    pub fn physical_inventory(&self) -> Money {
        self.physical_inventory
    }

    pub fn accounts_receivable(&self) -> Money {
        self.accounts_receivable
    }

    pub fn cash(&self) -> Money {
        self.cash
    }

    pub fn financial_assets(&self) -> Money {
        self.financial_assets
    }

    pub fn other(&self) -> Money {
        self.other
    }

    pub fn total(&self) -> Money {
        self.physical_inventory
            + self.accounts_receivable
            + self.cash
            + self.financial_assets
            + self.other
    }
}

impl Assets {
    pub fn new(current: CurrentAssets, non_current: NonCurrentAssets) -> Self {
        Self {
            current,
            non_current,
        }
    }

    pub fn current(&self) -> &CurrentAssets {
        &self.current
    }

    pub fn non_current(&self) -> &NonCurrentAssets {
        &self.non_current
    }

    pub fn total(&self) -> Money {
        self.current.total() + self.non_current.total()
    }
}

impl Equity {
    pub fn new(
        authorised_capital: Money,
        capital_surplus: Money,
        retained_earnings: Money,
        other: Money,
    ) -> Self {
        Self {
            authorised_capital,
            capital_surplus,
            retained_earnings,
            other,
        }
    }

    pub fn authorised_capital(&self) -> Money {
        self.authorised_capital
    }

    pub fn capital_surplus(&self) -> Money {
        self.capital_surplus
    }

    pub fn retained_earnings(&self) -> Money {
        self.retained_earnings
    }

    pub fn other(&self) -> Money {
        self.other
    }

    pub fn total(&self) -> Money {
        self.authorised_capital + self.capital_surplus + self.retained_earnings + self.other
    }
}

impl LongTermLiabilities {
    pub fn new(loans: Money, accounts_payable: Money, other: Money) -> Self {
        Self {
            loans,
            accounts_payable,
            other,
        }
    }

    pub fn loans(&self) -> Money {
        self.loans
    }

    pub fn accounts_payable(&self) -> Money {
        self.accounts_payable
    }

    pub fn other(&self) -> Money {
        self.other
    }

    pub fn total(&self) -> Money {
        self.loans + self.accounts_payable + self.other
    }
}

impl CurrentLiabilities {
    pub fn new(loans: Money, accounts_payable: Money, other: Money) -> Self {
        Self {
            loans,
            accounts_payable,
            other,
        }
    }

    pub fn loans(&self) -> Money {
        self.loans
    }

    pub fn accounts_payable(&self) -> Money {
        self.accounts_payable
    }

    pub fn other(&self) -> Money {
        self.other
    }

    pub fn total(&self) -> Money {
        self.loans + self.accounts_payable + self.other
    }
}

impl Liabilities {
    pub fn new(long_term: LongTermLiabilities, current: CurrentLiabilities) -> Liabilities {
        Self { long_term, current }
    }

    pub fn long_term(&self) -> &LongTermLiabilities {
        &self.long_term
    }

    pub fn current(&self) -> &CurrentLiabilities {
        &self.current
    }

    pub fn total(&self) -> Money {
        self.current.total() + self.long_term.total()
    }
}

impl BalanceReport {
    pub fn new(assets: Assets, equity: Equity, liabilities: Liabilities) -> Result<Self> {
        if assets.total() != equity.total() + liabilities.total() {
            return Err(eyre!(
                "Balance mismatch. Assets {} Equity {} Liabilities {}",
                assets.total(),
                equity.total(),
                liabilities.total()
            ));
        }
        Ok(Self {
            assets,
            equity,
            liabilities,
        })
    }

    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    pub fn equity(&self) -> &Equity {
        &self.equity
    }

    pub fn liabilities(&self) -> &Liabilities {
        &self.liabilities
    }
}
