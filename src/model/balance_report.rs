use super::money::Money;
use eyre::{Result, eyre};

// Необоротные активы.
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

pub struct Assets {
    current: CurrentAssets,
    non_current: NonCurrentAssets,
}

// Капитал
pub struct Equity {
    // Уставной капитал
    authorised_capital: Money,
    // Добавочный капитал
    capital_surplus: Money,
    // Нераспределённая прибыль (непокрытый убыток)
    retained_earnings: Money,
}

// Долгосрочные обязательства
pub struct LongTermLiabilities {
    // Кредиты и займы
    loans: Money,
    // Кредиторская задолженность
    accounts_receivable: Money,
    // Прочее
    other: Money,
}

// Краткосрочные обязательства
pub struct CurrentLiabilities {
    // Кредиты и займы
    loans: Money,
    // Кредиторская задолженность
    accounts_receivable: Money,
    // Прочее
    other: Money,
}

pub struct Liabilities {
    long_term: LongTermLiabilities,
    current: CurrentLiabilities,
}

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

    pub fn total(&self) -> Money {
        self.current.total() + self.non_current.total()
    }
}

impl Equity {
    pub fn new(
        authorised_capital: Money,
        capital_surplus: Money,
        retained_earnings: Money,
    ) -> Self {
        Self {
            authorised_capital,
            capital_surplus,
            retained_earnings,
        }
    }

    pub fn total(&self) -> Money {
        self.authorised_capital + self.capital_surplus + self.retained_earnings
    }
}

impl LongTermLiabilities {
    pub fn new(loans: Money, accounts_receivable: Money, other: Money) -> Self {
        Self {
            loans,
            accounts_receivable,
            other,
        }
    }
    pub fn total(&self) -> Money {
        self.loans + self.accounts_receivable + self.other
    }
}

impl CurrentLiabilities {
    pub fn new(loans: Money, accounts_receivable: Money, other: Money) -> Self {
        Self {
            loans,
            accounts_receivable,
            other,
        }
    }
    pub fn total(&self) -> Money {
        self.loans + self.accounts_receivable + self.other
    }
}

impl Liabilities {
    pub fn new(long_term: LongTermLiabilities, current: CurrentLiabilities) -> Liabilities {
        Self { long_term, current }
    }
    pub fn total(&self) -> Money {
        self.current.total() + self.long_term.total()
    }
}

impl BalanceReport {
    fn new(assets: Assets, equity: Equity, liabilities: Liabilities) -> Result<Self> {
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
}
