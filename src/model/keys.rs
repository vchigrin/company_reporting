use crate::model::Money;
use strum::VariantArray;
use strum_macros::{EnumString, IntoStaticStr, VariantArray};

pub trait GenericKeys:
    PartialEq
    + Clone
    + Copy
    + Default
    + std::fmt::Debug
    + std::str::FromStr<Err = strum::ParseError>
    + VariantArray
    + std::convert::Into<&'static str>
{
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, VariantArray, IntoStaticStr)]
pub enum BalanceKeys {
    // Основные средства
    FixedAssets,
    // Нематериальные активы
    NonMaterialAssets,
    // Финансовые вложения
    FinancialAssets,
    // Итого - внеоборотные активы
    TotalNonCurrentAssets,
    // Итого - оборотные активы
    TotalCurrentAssets,
    // Запасы
    PhysicalInventory,
    // Дебиторская задолженность
    AccountsReceivable,
    // Денежные средства и их эквиваленты
    Cash,
    // Уставной капитал
    AuthorisedCapital,
    // Добавочный капитал
    CapitalSurplus,
    // Нераспределённая прибыль (непокрытый убыток)
    RetainedEarnings,
    // итого капитал и резервы
    TotalEquity,
    // Кредиты и займы
    Loans,
    // Кредиторская задолженность
    AccountsPayable,

    // Итого долгосрочные обязательства
    TotalLongTermLiabilities,
    // Итого краткосрочные обязательства
    TotalCurrentLiabilities,
    TotalBalance,
    // Прочее
    #[default]
    Other,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, VariantArray, IntoStaticStr)]
pub enum IncomeKeys {
    // Выручка от реализации
    SalesRevenue,
    // Ceбестоимость продаж
    CostOfSales,
    // Валовая прибыль
    GrossProfit,
    // Коммерческие расходы
    CommercialExpenses,
    // Управленческие расходы
    ManagementExpenses,
    // Операционная прибыль
    OperationalProfit,
    // Прибыль от продаж. Актуально для РСБУ.
    ProfitFromSales,
    // Финансовые доходы
    FinancialIncome,
    // Финансовые расходы
    FinancialExpenses,
    // Прочие доходы
    #[default]
    OtherIncome,
    // Прочие расходы
    OtherExpenses,
    // Прибыль до налогообложения
    ProfitBeforeTax,
    // Налог на прибыль
    ProfitTax,
    // Чистая прибыль
    NetProfit,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, VariantArray, IntoStaticStr)]
pub enum CashFlowKeys {
    // Поступления от продаж
    SalesIncome,
    // Проценты полученные
    PercentIncome,
    // Оплата поставщикам
    SupplierExpenses,
    // Оплата труда
    SalariesExpenses,
    // Проценты уплаченные
    PercentExpenses,
    // Налоги
    TaxExpenses,
    // Продажа внеоборотных активов
    CapitalSales,
    // Приобретение внеоборотных активов
    CapitalExpenses,
    // Получение кредитов и займов
    RaisingLoans,
    // Эмиссия облигаций
    BondsIssuing,
    // Эмиссия акций
    SharesIssuing,
    // Возврат кредитов и займов
    ReturningLoans,
    // Погашение облигаций
    BondsRedemption,
    // Выкуп акций
    SharesBuyback,
    // Дивиденды
    Dividents,
    // Прочие доходы
    #[default]
    OtherIncome,
    // Прочие расходы
    OtherExpenses,
    TotalIncome,
    TotalExpenses,
    NetOperationalCashFlow,
    NetInvestmentCashFlow,
    NetFinancialCashFlow,
    NetTotalCashFlow,
}

impl GenericKeys for BalanceKeys {}
impl GenericKeys for IncomeKeys {}
impl GenericKeys for CashFlowKeys {}

#[derive(Debug, Clone)]
pub struct ParsedLineInfo<Keys: GenericKeys> {
    pub key: Keys,
    pub value: Money,
    pub original_line: String,
}
