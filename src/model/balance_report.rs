use super::money::Money;

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
