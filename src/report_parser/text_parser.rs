use crate::model::Money;
use crate::model::balance_report::{
    Assets, BalanceReport, CurrentAssets, CurrentLiabilities, Equity, Liabilities,
    LongTermLiabilities, NonCurrentAssets,
};
use crate::model::income_report::{
    FinancialSegment, GrossProfitSegment, IncomeReport, OperationalSegment,
};
use crate::report_parser::lines_classifier;
use crate::report_parser::{GenericKeys, ParsedLineInfo};
use color_print::cprintln;
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use strum_macros::{EnumString, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantArray)]
enum BalanceKeys {
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
    // Итого - активы
    // TotalAssets,
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
    // Прочее
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantArray)]
enum IncomeKeys {
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
    // Финансовые доходы
    FinancialIncome,
    // Финансовые расходы
    FinancialExpenses,
    // Прочие доходы
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

impl GenericKeys for BalanceKeys {}
impl GenericKeys for IncomeKeys {}

trait ParseHelper<Keys: GenericKeys> {
    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        delimeter_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo<Keys>]) -> Result<TParseResult>;

    fn new(analyzed_lines: Vec<ParsedLineInfo<Keys>>) -> Self;
}

struct BatchParserHelper<Keys: GenericKeys> {
    analyzed_lines: Vec<ParsedLineInfo<Keys>>,
    next_line_idx: usize,
}

impl<Keys: GenericKeys> BatchParserHelper<Keys> {
    fn next_line_index(&self) -> usize {
        self.next_line_idx
    }

    fn set_next_line_index(&mut self, new_val: usize) {
        self.next_line_idx = new_val
    }
}

impl<Keys: GenericKeys> ParseHelper<Keys> for BatchParserHelper<Keys> {
    fn new(analyzed_lines: Vec<ParsedLineInfo<Keys>>) -> Self {
        Self {
            analyzed_lines,
            next_line_idx: 0,
        }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        delimeter_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo<Keys>]) -> Result<TParseResult>,
    {
        if let Some(idx) = self.analyzed_lines[self.next_line_idx..]
            .iter()
            .position(|p| p.key == delimeter_key)
        {
            let result =
                cb(&self.analyzed_lines[self.next_line_idx..self.next_line_idx + idx + 1])?;
            self.next_line_idx += idx + 1;
            Ok(result)
        } else {
            Err(eyre!("Can not found line with {:?} key", delimeter_key))
        }
    }
}

struct InteractiveParserHelper<Keys: GenericKeys> {
    analyzed_lines: Vec<ParsedLineInfo<Keys>>,
    next_line_idx: usize,
}

impl<Keys: GenericKeys> InteractiveParserHelper<Keys> {
    fn read_string() -> Result<String> {
        let stdin = io::stdin();
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        Ok(buffer.trim().to_owned())
    }

    fn take_next_item(&mut self) -> Result<ParsedLineInfo<Keys>> {
        if self.next_line_idx < self.analyzed_lines.len() {
            let result = self.analyzed_lines[self.next_line_idx].clone();
            self.next_line_idx += 1;
            Ok(result)
        } else {
            Err(eyre!("End of parsed lines reached"))
        }
    }

    fn next_line_index(&self) -> usize {
        self.next_line_idx
    }

    fn set_next_line_index(&mut self, new_val: usize) {
        self.next_line_idx = new_val
    }
}

impl<Keys: GenericKeys> ParseHelper<Keys> for InteractiveParserHelper<Keys> {
    fn new(analyzed_lines: Vec<ParsedLineInfo<Keys>>) -> Self {
        Self {
            analyzed_lines,
            next_line_idx: 0,
        }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        delimeter_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo<Keys>]) -> Result<TParseResult>,
    {
        let mut filtered_lines = Vec::new();
        let mut cur_item = self.take_next_item()?;
        loop {
            cprintln!("\nLine <yellow>{}</yellow>", cur_item.original_line);
            cprintln!(
                "Key <green>{:?}</green>. Value <red>{}</red>",
                cur_item.key,
                cur_item.value
            );
            println!("k - change key. v - change value. Enter - Continue");
            let cmd = Self::read_string()?;
            match cmd.as_str() {
                "" => {
                    let should_finish = cur_item.key == delimeter_key;
                    filtered_lines.push(cur_item);
                    if should_finish {
                        break;
                    }
                    cur_item = self.take_next_item()?;
                }
                "v" => {
                    println!("Enter next value, in thousands or roubles:");
                    let sum_str = Self::read_string()?;
                    let sum = if let Ok(v) = i64::from_str(&sum_str) {
                        v
                    } else {
                        cprintln!("<red>Failed parse {}</red>", sum_str);
                        continue;
                    };
                    cur_item.value = Money::from_thousands(sum);
                }
                "k" => {
                    println!("Enter next key (allowed {:?}):", Keys::VARIANTS);
                    let key_str = Self::read_string()?;
                    let key = if let Ok(v) = Keys::from_str(&key_str) {
                        v
                    } else {
                        cprintln!("<red>Failed parse {}</red>", key_str);
                        continue;
                    };
                    cur_item.key = key;
                }
                _ => {
                    cprintln!("<red>Unknown command {}</red>", cmd);
                }
            }
        }
        cb(&filtered_lines)
    }
}

struct CombinedParseHelper<Keys: GenericKeys> {
    batch: BatchParserHelper<Keys>,
    interactive: InteractiveParserHelper<Keys>,
}

impl<Keys: GenericKeys> ParseHelper<Keys> for CombinedParseHelper<Keys> {
    fn new(analyzed_lines: Vec<ParsedLineInfo<Keys>>) -> Self {
        let batch = BatchParserHelper::<Keys>::new(analyzed_lines.clone());
        let interactive = InteractiveParserHelper::new(analyzed_lines);
        Self { batch, interactive }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        delimeter_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo<Keys>]) -> Result<TParseResult>,
    {
        match self.batch.parse_next(delimeter_key, cb) {
            Ok(batch_result) => {
                self.interactive
                    .set_next_line_index(self.batch.next_line_index());
                return Ok(batch_result);
            }
            Err(e) => {
                log::info!("Failed parse till {delimeter_key:?} in batch mode. Error {e}");
            }
        }
        let interactive_result = self.interactive.parse_next(delimeter_key, cb)?;
        self.batch
            .set_next_line_index(self.interactive.next_line_index());
        Ok(interactive_result)
    }
}

struct ParsedLineInfoCollector<Keys: GenericKeys> {
    key: Keys,
    result: Money,
}

impl<Keys: GenericKeys> ParsedLineInfoCollector<Keys> {
    fn new(key: Keys) -> Self {
        Self {
            key,
            result: Money::zero(),
        }
    }

    fn result(&self) -> Money {
        self.result
    }

    fn try_accept(&mut self, line: &ParsedLineInfo<Keys>) -> bool {
        if line.key == self.key {
            self.result += line.value;
            return true;
        }
        false
    }
}

fn parse_common_helper<Keys: GenericKeys>(
    report_lines: &[ParsedLineInfo<Keys>],
    all_collectors: &mut [&mut ParsedLineInfoCollector<Keys>],
) -> Result<()> {
    for item in report_lines {
        let mut accepted = false;
        for collector in &mut *all_collectors {
            if collector.try_accept(item) {
                accepted = true;
                break;
            }
        }
        if !accepted {
            return Err(eyre!("Unexpected key {:?}", item.key));
        }
    }
    Ok(())
}

pub struct ReportParser {
    line_to_balance_key: HashMap<String, BalanceKeys>,
    line_to_income_key: HashMap<String, IncomeKeys>,
}

impl ReportParser {
    pub fn new() -> Self {
        let mut line_to_balance_key = HashMap::new();
        // TODO: Move to permanent storage.
        line_to_balance_key.insert("основные средства".to_owned(), BalanceKeys::FixedAssets);
        line_to_balance_key.insert(
            "активы в форме права пользования".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert(
            "прочие внеоборотные финансовые активы".to_owned(),
            BalanceKeys::FinancialAssets,
        );
        line_to_balance_key.insert("отложенные налоговые активы".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "итого внеоборотные активы".to_owned(),
            BalanceKeys::TotalNonCurrentAssets,
        );
        line_to_balance_key.insert("запасы".to_owned(), BalanceKeys::PhysicalInventory);
        line_to_balance_key.insert(
            "торговая и прочая дебиторская задолженность".to_owned(),
            BalanceKeys::AccountsReceivable,
        );
        line_to_balance_key.insert("авансы выданные".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "переплата по налогу на прибыль".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert(
            "переплата по прочим налогам и ндс к возмещению".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert(
            "прочие оборотные финансовые активы".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert(
            "денежные средства и их эквиваленты".to_owned(),
            BalanceKeys::Cash,
        );
        line_to_balance_key.insert(
            "итого оборотные активы".to_owned(),
            BalanceKeys::TotalCurrentAssets,
        );
        //        line_to_balance_key.insert("итого активы".to_owned(), BalanceKeys::TotalAssets);
        line_to_balance_key.insert(
            "уставный капитал".to_owned(),
            BalanceKeys::AuthorisedCapital,
        );
        line_to_balance_key.insert(
            "нераспределенная прибыль".to_owned(),
            BalanceKeys::RetainedEarnings,
        );
        line_to_balance_key.insert(
            "итого капитал и резервы".to_owned(),
            BalanceKeys::TotalEquity,
        );
        line_to_balance_key.insert("процентные кредиты и займы".to_owned(), BalanceKeys::Loans);
        line_to_balance_key.insert(
            "отложенные налоговые обязательства".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert("обязательства по аренде".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "итого долгосрочные обязательства".to_owned(),
            BalanceKeys::TotalLongTermLiabilities,
        );
        line_to_balance_key.insert("кредиты и займы".to_owned(), BalanceKeys::Loans);
        line_to_balance_key.insert("обязательства по аренде".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "торговая и прочая кредиторская задолженность".to_owned(),
            BalanceKeys::AccountsPayable,
        );
        line_to_balance_key.insert("обязательства по договору".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "текущие обязательства по налогу на прибыль".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert(
            "кредиторская задолженность по прочим налогам".to_owned(),
            BalanceKeys::Other,
        );
        line_to_balance_key.insert("оценочные обязательства".to_owned(), BalanceKeys::Other);
        line_to_balance_key.insert(
            "итого краткосрочные обязательства".to_owned(),
            BalanceKeys::TotalCurrentLiabilities,
        );
        let mut line_to_income_key = HashMap::new();
        line_to_income_key.insert("выручка от реализации".to_owned(), IncomeKeys::SalesRevenue);
        line_to_income_key.insert(
            "себестоимость реализации".to_owned(),
            IncomeKeys::CostOfSales,
        );
        line_to_income_key.insert("валовая прибыль".to_owned(), IncomeKeys::GrossProfit);
        line_to_income_key.insert(
            "общехозяйственные и административные расходы".to_owned(),
            IncomeKeys::CommercialExpenses,
        );
        line_to_income_key.insert(
            "изменения в ожидаемых кредитных убытках, нетто".to_owned(),
            IncomeKeys::OtherIncome,
        );
        line_to_income_key.insert(
            "прочие операционные доходы".to_owned(),
            IncomeKeys::OtherExpenses,
        );
        line_to_income_key.insert(
            "прочие операционные доходы".to_owned(),
            IncomeKeys::OtherIncome,
        );
        line_to_income_key.insert(
            "операционная прибыль/(убыток)".to_owned(),
            IncomeKeys::OperationalProfit,
        );
        line_to_income_key.insert("финансовые доходы".to_owned(), IncomeKeys::FinancialIncome);
        line_to_income_key.insert(
            "финансовые расходы".to_owned(),
            IncomeKeys::FinancialExpenses,
        );
        line_to_income_key.insert("прочие расходы".to_owned(), IncomeKeys::OtherExpenses);
        line_to_income_key.insert(
            "прибыль/убыток до налогообложения".to_owned(),
            IncomeKeys::ProfitBeforeTax,
        );
        line_to_income_key.insert(
            "(расходы)/доходы по налогу за прибыль".to_owned(),
            IncomeKeys::ProfitTax,
        );
        line_to_income_key.insert(
            "итого совокупный доход/(убыток) за отчётный период".to_owned(),
            IncomeKeys::NetProfit,
        );
        Self {
            line_to_balance_key,
            line_to_income_key,
        }
    }

    fn parse_non_current_assets(
        non_current_assets_lines: &[ParsedLineInfo<BalanceKeys>],
    ) -> Result<NonCurrentAssets> {
        let mut fixed_assets = Money::zero();
        let mut non_material_assets = Money::zero();
        let mut financial_assets = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                BalanceKeys::FixedAssets => {
                    fixed_assets += item.value;
                }
                BalanceKeys::NonMaterialAssets => {
                    non_material_assets += item.value;
                }
                BalanceKeys::FinancialAssets => {
                    financial_assets += item.value;
                }
                BalanceKeys::Other => {
                    other += item.value;
                }
                BalanceKeys::TotalNonCurrentAssets => {
                    total = item.value;
                }
                _ => {
                    return Err(eyre!("Unexpected key {:?} in non-current assets", item.key));
                }
            }
        }
        let result =
            NonCurrentAssets::new(fixed_assets, non_material_assets, financial_assets, other);
        if result.total() != total {
            return Err(eyre!(
                "Balance mismatch in non-current assets. Calculated {} provided in report {}",
                result.total(),
                total
            ));
        }
        Ok(result)
    }

    fn parse_current_assets(report_lines: &[ParsedLineInfo<BalanceKeys>]) -> Result<CurrentAssets> {
        let mut physical_inventory = ParsedLineInfoCollector::new(BalanceKeys::PhysicalInventory);
        let mut accounts_receivable = ParsedLineInfoCollector::new(BalanceKeys::AccountsReceivable);
        let mut cash = ParsedLineInfoCollector::new(BalanceKeys::Cash);
        let mut financial_assets = ParsedLineInfoCollector::new(BalanceKeys::FinancialAssets);
        let mut other = ParsedLineInfoCollector::new(BalanceKeys::Other);
        let mut total = ParsedLineInfoCollector::new(BalanceKeys::TotalCurrentAssets);
        parse_common_helper(
            report_lines,
            &mut [
                &mut physical_inventory,
                &mut accounts_receivable,
                &mut cash,
                &mut financial_assets,
                &mut other,
                &mut total,
            ],
        )?;
        let result = CurrentAssets::new(
            physical_inventory.result(),
            accounts_receivable.result(),
            cash.result(),
            financial_assets.result(),
            other.result(),
        );

        if result.total() != total.result() {
            return Err(eyre!(
                "Balance mismatch in current assets. Calculated {} provided in report {}",
                result.total(),
                total.result()
            ));
        }
        Ok(result)
    }

    fn parse_equity(report_lines: &[ParsedLineInfo<BalanceKeys>]) -> Result<Equity> {
        let mut authorised_capital = ParsedLineInfoCollector::new(BalanceKeys::AuthorisedCapital);
        let mut capital_surplus = ParsedLineInfoCollector::new(BalanceKeys::CapitalSurplus);
        let mut retained_earnings = ParsedLineInfoCollector::new(BalanceKeys::RetainedEarnings);
        let mut other = ParsedLineInfoCollector::new(BalanceKeys::Other);
        let mut total = ParsedLineInfoCollector::new(BalanceKeys::TotalEquity);
        parse_common_helper(
            report_lines,
            &mut [
                &mut authorised_capital,
                &mut capital_surplus,
                &mut retained_earnings,
                &mut other,
                &mut total,
            ],
        )?;
        let result = Equity::new(
            authorised_capital.result(),
            capital_surplus.result(),
            retained_earnings.result(),
            other.result(),
        );
        if result.total() != total.result() {
            return Err(eyre!(
                "Balance mismatch in Equity. Calculated {} provided in report {}",
                result.total(),
                total.result()
            ));
        }
        Ok(result)
    }

    fn parse_long_term_liabilities(
        report_lines: &[ParsedLineInfo<BalanceKeys>],
    ) -> Result<LongTermLiabilities> {
        let mut loans = ParsedLineInfoCollector::new(BalanceKeys::Loans);
        let mut accounts_payable = ParsedLineInfoCollector::new(BalanceKeys::AccountsPayable);
        let mut other = ParsedLineInfoCollector::new(BalanceKeys::Other);
        let mut total = ParsedLineInfoCollector::new(BalanceKeys::TotalLongTermLiabilities);
        parse_common_helper(
            report_lines,
            &mut [&mut loans, &mut accounts_payable, &mut other, &mut total],
        )?;
        let result =
            LongTermLiabilities::new(loans.result(), accounts_payable.result(), other.result());
        if result.total() != total.result() {
            return Err(eyre!(
                "Balance mismatch in Long term liabilities. Calculated {} provided in report {}",
                result.total(),
                total.result()
            ));
        }
        Ok(result)
    }

    fn parse_current_liabilities(
        report_lines: &[ParsedLineInfo<BalanceKeys>],
    ) -> Result<CurrentLiabilities> {
        let mut loans = ParsedLineInfoCollector::new(BalanceKeys::Loans);
        let mut accounts_payable = ParsedLineInfoCollector::new(BalanceKeys::AccountsPayable);
        let mut other = ParsedLineInfoCollector::new(BalanceKeys::Other);
        let mut total = ParsedLineInfoCollector::new(BalanceKeys::TotalCurrentLiabilities);
        parse_common_helper(
            report_lines,
            &mut [&mut loans, &mut accounts_payable, &mut other, &mut total],
        )?;

        let result =
            CurrentLiabilities::new(loans.result(), accounts_payable.result(), other.result());
        if result.total() != total.result() {
            return Err(eyre!(
                "Balance mismatch in Current term liabilities. Calculated {} provided in report {}",
                result.total(),
                total.result()
            ));
        }
        Ok(result)
    }

    pub fn parse_balance_report_batch(&self, page_lines: &[String]) -> Result<BalanceReport> {
        let parsed_lines =
            lines_classifier::classify_lines::<BalanceKeys>(page_lines, &self.line_to_balance_key)?;

        self.parse_balance_report_generic::<BatchParserHelper<BalanceKeys>>(parsed_lines)
    }

    pub fn parse_balance_report_interactive(&self, page_lines: &[String]) -> Result<BalanceReport> {
        let parsed_lines =
            lines_classifier::classify_lines::<BalanceKeys>(page_lines, &self.line_to_balance_key)?;

        self.parse_balance_report_generic::<CombinedParseHelper<BalanceKeys>>(parsed_lines)
    }

    fn parse_balance_report_generic<Helper>(
        &self,
        parsed_lines: Vec<ParsedLineInfo<BalanceKeys>>,
    ) -> Result<BalanceReport>
    where
        Helper: ParseHelper<BalanceKeys>,
    {
        let mut helper = Helper::new(parsed_lines);

        let non_current_assets = helper.parse_next(
            BalanceKeys::TotalNonCurrentAssets,
            &Self::parse_non_current_assets,
        )?;
        log::info!("Parsed non-current assets OK: {:?}", non_current_assets);

        let current_assets =
            helper.parse_next(BalanceKeys::TotalCurrentAssets, &Self::parse_current_assets)?;
        log::info!("Parsed current assets OK: {:?}", current_assets);

        let equity = helper.parse_next(BalanceKeys::TotalEquity, &Self::parse_equity)?;
        log::info!("Parsed equity OK: {:?}", equity);

        let long_term_liabilities = helper.parse_next(
            BalanceKeys::TotalLongTermLiabilities,
            &Self::parse_long_term_liabilities,
        )?;
        log::info!("Parsed long term liabilities OK: {:?}", equity);

        let current_liabilities = helper.parse_next(
            BalanceKeys::TotalCurrentLiabilities,
            &Self::parse_current_liabilities,
        )?;
        log::info!("Parsed current liabilities OK: {:?}", equity);

        BalanceReport::new(
            Assets::new(current_assets, non_current_assets),
            equity,
            Liabilities::new(long_term_liabilities, current_liabilities),
        )
    }

    fn parse_income_report_generic<Helper>(&self, page_lines: &[String]) -> Result<IncomeReport>
    where
        Helper: ParseHelper<IncomeKeys>,
    {
        let parsed_lines =
            lines_classifier::classify_lines::<IncomeKeys>(page_lines, &self.line_to_income_key)?;

        let mut helper = Helper::new(parsed_lines);

        let gross_profit_segment =
            helper.parse_next(IncomeKeys::GrossProfit, &Self::parse_gross_profit)?;
        log::info!("Parsed gross income OK: {:?}", gross_profit_segment);

        let operational_segment = helper.parse_next(
            IncomeKeys::OperationalProfit,
            &Self::parse_operational_segment,
        )?;
        log::info!("Parsed operationsl segment OK: {:?}", operational_segment);

        let financial_segment =
            helper.parse_next(IncomeKeys::ProfitBeforeTax, &Self::parse_financial_segment)?;
        log::info!("Parsed financial segment OK: {:?}", financial_segment);

        let profit_tax = helper.parse_next(IncomeKeys::ProfitTax, &Self::parse_profit_tax)?;
        log::info!("Parsed profit tax OK: {:?}", profit_tax);

        Ok(IncomeReport::new(
            gross_profit_segment,
            operational_segment,
            financial_segment,
            profit_tax,
        ))
    }

    pub fn parse_income_report_batch(&self, page_lines: &[String]) -> Result<IncomeReport> {
        self.parse_income_report_generic::<BatchParserHelper<IncomeKeys>>(page_lines)
    }

    pub fn parse_income_report_interactive(&self, page_lines: &[String]) -> Result<IncomeReport> {
        self.parse_income_report_generic::<CombinedParseHelper<IncomeKeys>>(page_lines)
    }

    fn parse_gross_profit(
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<GrossProfitSegment> {
        let mut sales_revenue = ParsedLineInfoCollector::new(IncomeKeys::SalesRevenue);
        let mut cost_of_sales = ParsedLineInfoCollector::new(IncomeKeys::CostOfSales);
        let mut gross_profit = ParsedLineInfoCollector::new(IncomeKeys::GrossProfit);
        parse_common_helper(
            report_lines,
            &mut [&mut sales_revenue, &mut cost_of_sales, &mut gross_profit],
        )?;

        let result = GrossProfitSegment::new(sales_revenue.result(), cost_of_sales.result())?;
        if result.gross_profit() != gross_profit.result() {
            return Err(eyre!(
                "Income mismatch in gross profit. Calculated {} provided in report {}",
                result.gross_profit(),
                gross_profit.result()
            ));
        }
        Ok(result)
    }

    fn parse_operational_segment(
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<OperationalSegment> {
        let mut commercial_expenses = ParsedLineInfoCollector::new(IncomeKeys::CommercialExpenses);
        let mut management_expenses = ParsedLineInfoCollector::new(IncomeKeys::ManagementExpenses);
        let mut other_income = ParsedLineInfoCollector::new(IncomeKeys::OtherIncome);
        let mut other_expenses = ParsedLineInfoCollector::new(IncomeKeys::OtherExpenses);
        parse_common_helper(
            report_lines,
            &mut [
                &mut commercial_expenses,
                &mut management_expenses,
                &mut other_income,
                &mut other_expenses,
            ],
        )?;

        let result = OperationalSegment::new(
            commercial_expenses.result(),
            management_expenses.result(),
            other_income.result(),
            other_expenses.result(),
        );
        // TODO(vchigrin): We need GrossProfit to validate operational segment...
        Ok(result)
    }

    fn parse_financial_segment(
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<FinancialSegment> {
        let mut financial_income = ParsedLineInfoCollector::new(IncomeKeys::FinancialIncome);
        let mut financial_expenses = ParsedLineInfoCollector::new(IncomeKeys::FinancialExpenses);
        parse_common_helper(
            report_lines,
            &mut [&mut financial_income, &mut financial_expenses],
        )?;
        let result = FinancialSegment::new(financial_income.result(), financial_expenses.result());
        // TODO(vchigrin): We need GrossProfit to validate operational segment...
        Ok(result)
    }

    fn parse_profit_tax(report_lines: &[ParsedLineInfo<IncomeKeys>]) -> Result<Money> {
        let mut profit_tax = ParsedLineInfoCollector::new(IncomeKeys::ProfitTax);
        parse_common_helper(report_lines, &mut [&mut profit_tax])?;
        Ok(profit_tax.result())
    }
}
