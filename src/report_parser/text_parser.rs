use crate::model::Money;
use crate::model::balance_report::{
    Assets, BalanceReport, CurrentAssets, CurrentLiabilities, Equity, Liabilities,
    LongTermLiabilities, NonCurrentAssets,
};
use crate::model::income_report::IncomeReport;
use color_print::cprintln;
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use strum::VariantArray;
use strum_macros::{EnumString, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantArray)]
enum Keys {
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

#[derive(Debug, Clone)]
struct ParsedLineInfo {
    key: Keys,
    value: Money,
    original_line: String,
}

trait ParseHelper {
    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        total_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo]) -> Result<TParseResult>;

    fn new(analyzed_lines: Vec<ParsedLineInfo>) -> Self;
}

struct BatchParserHelper {
    analyzed_lines: Vec<ParsedLineInfo>,
    next_line_idx: usize,
}

impl BatchParserHelper {
    fn next_line_index(&self) -> usize {
        self.next_line_idx
    }

    fn set_next_line_index(&mut self, new_val: usize) {
        self.next_line_idx = new_val
    }
}

impl ParseHelper for BatchParserHelper {
    fn new(analyzed_lines: Vec<ParsedLineInfo>) -> Self {
        Self {
            analyzed_lines,
            next_line_idx: 0,
        }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        total_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo]) -> Result<TParseResult>,
    {
        if let Some(idx) = self.analyzed_lines[self.next_line_idx..]
            .iter()
            .position(|p| p.key == total_key)
        {
            let result =
                cb(&self.analyzed_lines[self.next_line_idx..self.next_line_idx + idx + 1])?;
            self.next_line_idx += idx + 1;
            Ok(result)
        } else {
            Err(eyre!("Can not found line with {:?} key", total_key))
        }
    }
}

struct InteractiveParserHelper {
    analyzed_lines: Vec<ParsedLineInfo>,
    next_line_idx: usize,
}

impl InteractiveParserHelper {
    fn read_string() -> Result<String> {
        let stdin = io::stdin();
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        Ok(buffer.trim().to_owned())
    }

    fn take_next_item(&mut self) -> Result<ParsedLineInfo> {
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

impl ParseHelper for InteractiveParserHelper {
    fn new(analyzed_lines: Vec<ParsedLineInfo>) -> Self {
        Self {
            analyzed_lines,
            next_line_idx: 0,
        }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        total_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo]) -> Result<TParseResult>,
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
                    let should_finish = cur_item.key == total_key;
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

struct CombinedParseHelper {
    batch: BatchParserHelper,
    interactive: InteractiveParserHelper,
}

impl ParseHelper for CombinedParseHelper {
    fn new(analyzed_lines: Vec<ParsedLineInfo>) -> Self {
        let batch = BatchParserHelper::new(analyzed_lines.clone());
        let interactive = InteractiveParserHelper::new(analyzed_lines);
        Self { batch, interactive }
    }

    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        total_key: Keys,
        cb: &TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo]) -> Result<TParseResult>,
    {
        match self.batch.parse_next(total_key, cb) {
            Ok(batch_result) => {
                self.interactive
                    .set_next_line_index(self.batch.next_line_index());
                return Ok(batch_result);
            }
            Err(e) => {
                log::info!("Failed parse till {total_key:?} in batch mode. Error {e}");
            }
        }
        let interactive_result = self.interactive.parse_next(total_key, cb)?;
        self.batch
            .set_next_line_index(self.interactive.next_line_index());
        Ok(interactive_result)
    }
}

pub struct ReportParser {
    line_to_key: HashMap<String, Keys>,
}

impl ReportParser {
    pub fn new() -> Self {
        let mut line_to_key = HashMap::new();
        // TODO: Move to permanent storage.
        line_to_key.insert("основные средства".to_owned(), Keys::FixedAssets);
        line_to_key.insert("активы в форме права пользования".to_owned(), Keys::Other);
        line_to_key.insert(
            "прочие внеоборотные финансовые активы".to_owned(),
            Keys::FinancialAssets,
        );
        line_to_key.insert("отложенные налоговые активы".to_owned(), Keys::Other);
        line_to_key.insert(
            "итого внеоборотные активы".to_owned(),
            Keys::TotalNonCurrentAssets,
        );
        line_to_key.insert("запасы".to_owned(), Keys::PhysicalInventory);
        line_to_key.insert(
            "торговая и прочая дебиторская задолженность".to_owned(),
            Keys::AccountsReceivable,
        );
        line_to_key.insert("авансы выданные".to_owned(), Keys::Other);
        line_to_key.insert("переплата по налогу на прибыль".to_owned(), Keys::Other);
        line_to_key.insert(
            "переплата по прочим налогам и ндс к возмещению".to_owned(),
            Keys::Other,
        );
        line_to_key.insert("прочие оборотные финансовые активы".to_owned(), Keys::Other);
        line_to_key.insert("денежные средства и их эквиваленты".to_owned(), Keys::Cash);
        line_to_key.insert(
            "итого оборотные активы".to_owned(),
            Keys::TotalCurrentAssets,
        );
        //        line_to_key.insert("итого активы".to_owned(), Keys::TotalAssets);
        line_to_key.insert("уставный капитал".to_owned(), Keys::AuthorisedCapital);
        line_to_key.insert(
            "нераспределенная прибыль".to_owned(),
            Keys::RetainedEarnings,
        );
        line_to_key.insert("итого капитал и резервы".to_owned(), Keys::TotalEquity);
        line_to_key.insert("процентные кредиты и займы".to_owned(), Keys::Loans);
        line_to_key.insert("отложенные налоговые обязательства".to_owned(), Keys::Other);
        line_to_key.insert("обязательства по аренде".to_owned(), Keys::Other);
        line_to_key.insert(
            "итого долгосрочные обязательства".to_owned(),
            Keys::TotalLongTermLiabilities,
        );
        line_to_key.insert("кредиты и займы".to_owned(), Keys::Loans);
        line_to_key.insert("обязательства по аренде".to_owned(), Keys::Other);
        line_to_key.insert(
            "торговая и прочая кредиторская задолженность".to_owned(),
            Keys::AccountsPayable,
        );
        line_to_key.insert("обязательства по договору".to_owned(), Keys::Other);
        line_to_key.insert(
            "текущие обязательства по налогу на прибыль".to_owned(),
            Keys::Other,
        );
        line_to_key.insert(
            "кредиторская задолженность по прочим налогам".to_owned(),
            Keys::Other,
        );
        line_to_key.insert("оценочные обязательства".to_owned(), Keys::Other);
        line_to_key.insert(
            "итого краткосрочные обязательства".to_owned(),
            Keys::TotalCurrentLiabilities,
        );
        Self { line_to_key }
    }

    fn parse_money(line: &str) -> Result<Money> {
        // Sometimes tesseract add garbage '.' characters.
        let filtered: String = line
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '.')
            .collect();
        if filtered.is_empty() {
            return Err(eyre!("Can not parse {} as money", line));
        }
        if filtered == "-" {
            return Ok(Money::zero());
        }
        if filtered.starts_with('(') && filtered.ends_with(')') {
            let val = i64::from_str(filtered.trim_matches(|c| c == '(' || c == ')'))?;
            Ok(Money::from_thousands(-val))
        } else {
            let val = i64::from_str(&filtered)?;
            Ok(Money::from_thousands(val))
        }
    }

    fn parse_lines(&self, page_lines: &[String]) -> Result<Vec<ParsedLineInfo>> {
        let mut result = Vec::new();
        for line in page_lines {
            let tokens: Vec<&str> = line.split("  ").filter(|p| !p.is_empty()).collect();
            if tokens.len() != 3 && tokens.len() != 4 {
                log::info!("Skipping non-report line {:?}", tokens);
                continue;
            }
            let line_token: String = tokens[0]
                .to_lowercase()
                .chars()
                .map(|c| {
                    // Leave only lowercase Russian, to strip OCR artifacts.
                    if ('а'..='я').contains(&c) {
                        return c;
                    }
                    ' '
                })
                .collect();
            if let Some(key) = self.line_to_key.get(line_token.trim()) {
                // Last element is the value of previous period.
                // One before last - for current period.
                // Two before last - optional reference to additional info in report.
                let current_value_str = tokens[tokens.len() - 2];
                let money = match Self::parse_money(current_value_str) {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(eyre!("Error {} on line {}", e, line));
                    }
                };
                result.push(ParsedLineInfo {
                    key: *key,
                    value: money,
                    original_line: line.to_owned(),
                });
            } else {
                log::warn!("Unknown line {:?}", line_token);
            }
        }
        Ok(result)
    }

    fn parse_non_current_assets(
        non_current_assets_lines: &[ParsedLineInfo],
    ) -> Result<NonCurrentAssets> {
        let mut fixed_assets = Money::zero();
        let mut non_material_assets = Money::zero();
        let mut financial_assets = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                Keys::FixedAssets => {
                    fixed_assets += item.value;
                }
                Keys::NonMaterialAssets => {
                    non_material_assets += item.value;
                }
                Keys::FinancialAssets => {
                    financial_assets += item.value;
                }
                Keys::Other => {
                    other += item.value;
                }
                Keys::TotalNonCurrentAssets => {
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

    fn parse_current_assets(non_current_assets_lines: &[ParsedLineInfo]) -> Result<CurrentAssets> {
        let mut physical_inventory = Money::zero();
        let mut accounts_receivable = Money::zero();
        let mut cash = Money::zero();
        let mut financial_assets = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                Keys::PhysicalInventory => {
                    physical_inventory += item.value;
                }
                Keys::AccountsReceivable => {
                    accounts_receivable += item.value;
                }
                Keys::Cash => {
                    cash += item.value;
                }
                Keys::FinancialAssets => {
                    financial_assets += item.value;
                }
                Keys::Other => {
                    other += item.value;
                }
                Keys::TotalCurrentAssets => {
                    total = item.value;
                }
                _ => {
                    return Err(eyre!("Unexpected key {:?} in current assets", item.key));
                }
            }
        }
        let result = CurrentAssets::new(
            physical_inventory,
            accounts_receivable,
            cash,
            financial_assets,
            other,
        );

        if result.total() != total {
            return Err(eyre!(
                "Balance mismatch in current assets. Calculated {} provided in report {}",
                result.total(),
                total
            ));
        }
        Ok(result)
    }

    fn parse_equity(non_current_assets_lines: &[ParsedLineInfo]) -> Result<Equity> {
        let mut authorised_capital = Money::zero();
        let mut capital_surplus = Money::zero();
        let mut retained_earnings = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                Keys::AuthorisedCapital => {
                    authorised_capital += item.value;
                }
                Keys::CapitalSurplus => {
                    capital_surplus += item.value;
                }
                Keys::RetainedEarnings => {
                    retained_earnings += item.value;
                }
                Keys::Other => {
                    other += item.value;
                }
                Keys::TotalEquity => {
                    total = item.value;
                }
                _ => {
                    return Err(eyre!("Unexpected key {:?} in equity", item.key));
                }
            }
        }
        let result = Equity::new(
            authorised_capital,
            capital_surplus,
            retained_earnings,
            other,
        );
        if result.total() != total {
            return Err(eyre!(
                "Balance mismatch in Equity. Calculated {} provided in report {}",
                result.total(),
                total
            ));
        }
        Ok(result)
    }

    fn parse_long_term_liabilities(
        non_current_assets_lines: &[ParsedLineInfo],
    ) -> Result<LongTermLiabilities> {
        let mut loans = Money::zero();
        let mut accounts_payable = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                Keys::Loans => {
                    loans += item.value;
                }
                Keys::AccountsPayable => {
                    accounts_payable += item.value;
                }
                Keys::Other => {
                    other += item.value;
                }
                Keys::TotalLongTermLiabilities => {
                    total = item.value;
                }
                _ => {
                    return Err(eyre!(
                        "Unexpected key {:?} in long term liabilities",
                        item.key
                    ));
                }
            }
        }
        let result = LongTermLiabilities::new(loans, accounts_payable, other);
        if result.total() != total {
            return Err(eyre!(
                "Balance mismatch in Long term liabilities. Calculated {} provided in report {}",
                result.total(),
                total
            ));
        }
        Ok(result)
    }

    fn parse_current_liabilities(
        non_current_assets_lines: &[ParsedLineInfo],
    ) -> Result<CurrentLiabilities> {
        let mut loans = Money::zero();
        let mut accounts_payable = Money::zero();
        let mut other = Money::zero();
        let mut total = Money::zero();
        for item in non_current_assets_lines {
            match item.key {
                Keys::Loans => {
                    loans += item.value;
                }
                Keys::AccountsPayable => {
                    accounts_payable += item.value;
                }
                Keys::Other => {
                    other += item.value;
                }
                Keys::TotalCurrentLiabilities => {
                    total = item.value;
                }
                _ => {
                    return Err(eyre!(
                        "Unexpected key {:?} in long term liabilities",
                        item.key
                    ));
                }
            }
        }
        let result = CurrentLiabilities::new(loans, accounts_payable, other);
        if result.total() != total {
            return Err(eyre!(
                "Balance mismatch in Current term liabilities. Calculated {} provided in report {}",
                result.total(),
                total
            ));
        }
        Ok(result)
    }

    pub fn parse_balance_report_batch(&self, page_lines: &[String]) -> Result<BalanceReport> {
        self.parse_balance_report_generic::<BatchParserHelper>(page_lines)
    }

    pub fn parse_balance_report_interactive(&self, page_lines: &[String]) -> Result<BalanceReport> {
        self.parse_balance_report_generic::<CombinedParseHelper>(page_lines)
    }

    fn parse_balance_report_generic<Helper>(&self, page_lines: &[String]) -> Result<BalanceReport>
    where
        Helper: ParseHelper,
    {
        let parsed_lines = self.parse_lines(page_lines)?;

        let mut helper = Helper::new(parsed_lines);

        let non_current_assets =
            helper.parse_next(Keys::TotalNonCurrentAssets, &Self::parse_non_current_assets)?;
        log::info!("Parsed non-current assets OK: {:?}", non_current_assets);

        let current_assets =
            helper.parse_next(Keys::TotalCurrentAssets, &Self::parse_current_assets)?;
        log::info!("Parsed current assets OK: {:?}", current_assets);

        let equity = helper.parse_next(Keys::TotalEquity, &Self::parse_equity)?;
        log::info!("Parsed equity OK: {:?}", equity);

        let long_term_liabilities = helper.parse_next(
            Keys::TotalLongTermLiabilities,
            &Self::parse_long_term_liabilities,
        )?;
        log::info!("Parsed long term liabilities OK: {:?}", equity);

        let current_liabilities = helper.parse_next(
            Keys::TotalCurrentLiabilities,
            &Self::parse_current_liabilities,
        )?;
        log::info!("Parsed current liabilities OK: {:?}", equity);

        BalanceReport::new(
            Assets::new(current_assets, non_current_assets),
            equity,
            Liabilities::new(long_term_liabilities, current_liabilities),
        )
    }

    pub fn parse_income_report(&self, page_lines: &[String]) -> Result<IncomeReport> {
        unimplemented!();
    }
}
