use crate::model::Money;
use crate::model::balance_report::{
    Assets, BalanceReport, CurrentAssets, CurrentLiabilities, Equity, Liabilities,
    LongTermLiabilities, NonCurrentAssets,
};
use crate::model::income_report::IncomeReport;
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
struct ParsedLineInfo {
    key: Keys,
    value: Money,
}

trait ParseHelper {
    fn parse_next<TParseCb, TParseResult>(
        &mut self,
        total_key: Keys,
        cb: TParseCb,
    ) -> Result<TParseResult>
    where
        TParseCb: Fn(&[ParsedLineInfo]) -> Result<TParseResult>;

    fn new(analyzed_lines: Vec<ParsedLineInfo>) -> Self;
}

struct BatchParserHelper {
    analyzed_lines: Vec<ParsedLineInfo>,
    next_line_idx: usize,
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
        cb: TParseCb,
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
            return Ok(result);
        } else {
            return Err(eyre!("Can not found line with {:?} key", total_key));
        }
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
            return Ok(Money::from_thousands(-val));
        } else {
            let val = i64::from_str(&filtered)?;
            return Ok(Money::from_thousands(val));
        }
    }

    fn parse_lines(&self, page_lines: &[String]) -> Result<Vec<ParsedLineInfo>> {
        let mut result = Vec::new();
        for line in page_lines {
            let tokens: Vec<&str> = line.split("  ").filter(|p| p.len() > 0).collect();
            if tokens.len() != 3 && tokens.len() != 4 {
                log::info!("Skipping non-report line {:?}", tokens);
                continue;
            }
            let line_token: String = tokens[0]
                .to_lowercase()
                .chars()
                .map(|c| {
                    // Leave only lowercase Russian, to strip OCR artifacts.
                    if c >= 'а' && c <= 'я' {
                        return c;
                    }
                    return ' ';
                })
                .collect();
            if let Some(key) = self.line_to_key.get(line_token.trim()) {
                // Last element is the value of previous period.
                // One before last - for current period.
                // Two before last - optional reference to additional info in report.
                let current_value_str = tokens[tokens.len() - 2];
                let money: Money;
                match Self::parse_money(current_value_str) {
                    Ok(m) => {
                        money = m;
                    }
                    Err(e) => {
                        return Err(eyre!("Error {} on line {}", e, line));
                    }
                }
                result.push(ParsedLineInfo {
                    key: *key,
                    value: money,
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

    fn parse_balance_report_generic<Helper>(&self, page_lines: &[String]) -> Result<BalanceReport>
    where
        Helper: ParseHelper,
    {
        let parsed_lines = self.parse_lines(page_lines)?;

        let mut helper = Helper::new(parsed_lines);

        let non_current_assets =
            helper.parse_next(Keys::TotalNonCurrentAssets, Self::parse_non_current_assets)?;
        log::info!("Parsed non-current assets OK: {:?}", non_current_assets);

        let current_assets =
            helper.parse_next(Keys::TotalCurrentAssets, Self::parse_current_assets)?;
        log::info!("Parsed current assets OK: {:?}", current_assets);

        let equity = helper.parse_next(Keys::TotalEquity, Self::parse_equity)?;
        log::info!("Parsed equity OK: {:?}", equity);

        let long_term_liabilities = helper.parse_next(
            Keys::TotalLongTermLiabilities,
            Self::parse_long_term_liabilities,
        )?;
        log::info!("Parsed long term liabilities OK: {:?}", equity);

        let current_liabilities = helper.parse_next(
            Keys::TotalCurrentLiabilities,
            Self::parse_current_liabilities,
        )?;
        log::info!("Parsed current lities OK: {:?}", equity);

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
