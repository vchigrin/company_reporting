use super::lines_classifier;
use crate::model::Money;
use crate::model::balance_report::{
    Assets, BalanceReport, CurrentAssets, CurrentLiabilities, Equity, Liabilities,
    LongTermLiabilities, NonCurrentAssets,
};
use crate::model::income_report::{
    FinancialSegment, GrossProfitSegment, IncomeReport, OperationalSegment,
};
use crate::model::{BalanceKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
use crate::ui::interactive_lines_editor::{EditResult, InteractiveLinesEditor};
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::rc::Rc;

struct BatchParserHelper<'a, Keys: GenericKeys> {
    analyzed_lines: &'a Vec<ParsedLineInfo<Keys>>,
    next_line_idx: usize,
}

impl<'a, Keys: GenericKeys> BatchParserHelper<'a, Keys> {
    fn new(analyzed_lines: &'a Vec<ParsedLineInfo<Keys>>) -> Self {
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

pub struct ReportParser {}

impl ReportParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make_default_balance_keys_classifier()
    -> Rc<dyn lines_classifier::KeyClassifier<BalanceKeys>> {
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
        Rc::new(lines_classifier::MapKeyClasifier::new(line_to_balance_key))
    }

    pub fn make_default_income_keys_classifier()
    -> Rc<dyn lines_classifier::KeyClassifier<IncomeKeys>> {
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
        Rc::new(lines_classifier::MapKeyClasifier::new(line_to_income_key))
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

    pub fn parse_balance_report_interactive(
        &self,
        parsed_lines: Vec<ParsedLineInfo<BalanceKeys>>,
    ) -> Result<Option<Vec<ParsedLineInfo<BalanceKeys>>>> {
        let mut editor = InteractiveLinesEditor::new(parsed_lines);
        loop {
            let edit_result = editor.run_editor()?;
            if edit_result == EditResult::CloseWithoutSaving {
                return Ok(None);
            }
            match self.parse_balance_report_batch(editor.result_lines()) {
                Ok(_) => {
                    return Ok(Some(editor.result_lines().clone()));
                }
                Err(err) => {
                    editor.set_error_line(err.to_string());
                }
            }
        }
    }

    pub fn parse_balance_report_batch(
        &self,
        parsed_lines: &Vec<ParsedLineInfo<BalanceKeys>>,
    ) -> Result<BalanceReport> {
        let mut helper = BatchParserHelper::new(parsed_lines);

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

    pub fn parse_income_report_batch(
        &self,
        parsed_lines: &Vec<ParsedLineInfo<IncomeKeys>>,
    ) -> Result<IncomeReport> {
        let mut helper = BatchParserHelper::<IncomeKeys>::new(parsed_lines);

        let gross_profit_segment =
            helper.parse_next(IncomeKeys::GrossProfit, &Self::parse_gross_profit)?;
        log::info!("Parsed gross income OK: {:?}", gross_profit_segment);

        let gross_profit = gross_profit_segment.gross_profit();
        let operational_segment = helper
            .parse_next(IncomeKeys::OperationalProfit, &|report_lines| {
                Self::parse_operational_segment(gross_profit, report_lines)
            })?;
        log::info!("Parsed operational segment OK: {:?}", operational_segment);

        let operational_profit = operational_segment.operational_profit(gross_profit);
        let financial_segment = helper
            .parse_next(IncomeKeys::ProfitBeforeTax, &|report_lines| {
                Self::parse_financial_segment(operational_profit, report_lines)
            })?;
        log::info!("Parsed financial segment OK: {:?}", financial_segment);

        let profit_before_tax = financial_segment.profit_before_tax(operational_profit);
        let profit_tax = helper.parse_next(IncomeKeys::NetProfit, &|report_lines| {
            Self::parse_profit_tax(profit_before_tax, report_lines)
        })?;
        log::info!("Parsed profit tax OK: {:?}", profit_tax);

        Ok(IncomeReport::new(
            gross_profit_segment,
            operational_segment,
            financial_segment,
            profit_tax,
        ))
    }

    pub fn parse_income_report_interactive(
        &self,
        parsed_lines: Vec<ParsedLineInfo<IncomeKeys>>,
    ) -> Result<Option<Vec<ParsedLineInfo<IncomeKeys>>>> {
        let mut editor = InteractiveLinesEditor::new(parsed_lines);
        loop {
            let edit_result = editor.run_editor()?;
            if edit_result == EditResult::CloseWithoutSaving {
                return Ok(None);
            }
            match self.parse_income_report_batch(editor.result_lines()) {
                Ok(_) => {
                    return Ok(Some(editor.result_lines().clone()));
                }
                Err(err) => {
                    editor.set_error_line(err.to_string());
                }
            }
        }
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
        gross_profit: Money,
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<OperationalSegment> {
        let mut commercial_expenses = ParsedLineInfoCollector::new(IncomeKeys::CommercialExpenses);
        let mut management_expenses = ParsedLineInfoCollector::new(IncomeKeys::ManagementExpenses);
        let mut other_income = ParsedLineInfoCollector::new(IncomeKeys::OtherIncome);
        let mut other_expenses = ParsedLineInfoCollector::new(IncomeKeys::OtherExpenses);
        let mut operational_profit = ParsedLineInfoCollector::new(IncomeKeys::OperationalProfit);
        parse_common_helper(
            report_lines,
            &mut [
                &mut commercial_expenses,
                &mut management_expenses,
                &mut other_income,
                &mut other_expenses,
                &mut operational_profit,
            ],
        )?;

        let result = OperationalSegment::new(
            commercial_expenses.result(),
            management_expenses.result(),
            other_income.result(),
            other_expenses.result(),
        )?;
        let calculated_profit = result.operational_profit(gross_profit);
        if calculated_profit != operational_profit.result() {
            return Err(eyre!(
                "Income mismatch in operational profit. Calculated {} provided in report {}",
                calculated_profit.to_string(),
                operational_profit.result().to_string()
            ));
        }
        Ok(result)
    }

    fn parse_financial_segment(
        operational_profit: Money,
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<FinancialSegment> {
        let mut financial_income = ParsedLineInfoCollector::new(IncomeKeys::FinancialIncome);
        let mut financial_expenses = ParsedLineInfoCollector::new(IncomeKeys::FinancialExpenses);
        let mut profit_before_tax = ParsedLineInfoCollector::new(IncomeKeys::ProfitBeforeTax);
        parse_common_helper(
            report_lines,
            &mut [
                &mut financial_income,
                &mut financial_expenses,
                &mut profit_before_tax,
            ],
        )?;
        let result = FinancialSegment::new(financial_income.result(), financial_expenses.result())?;
        let calculated_profit = result.profit_before_tax(operational_profit);
        if calculated_profit != profit_before_tax.result() {
            return Err(eyre!(
                "Income mismatch in profit before tax. Calculated {} provided in report {}",
                calculated_profit.to_string(),
                profit_before_tax.result().to_string()
            ));
        }
        Ok(result)
    }

    fn parse_profit_tax(
        profit_before_tax: Money,
        report_lines: &[ParsedLineInfo<IncomeKeys>],
    ) -> Result<Money> {
        let mut profit_tax = ParsedLineInfoCollector::new(IncomeKeys::ProfitTax);
        let mut net_profit = ParsedLineInfoCollector::new(IncomeKeys::NetProfit);
        parse_common_helper(report_lines, &mut [&mut profit_tax, &mut net_profit])?;
        // NOTE: seems, profit_tax can be positive in case when company get
        // financial losses before tax.
        let calculated_profit = profit_before_tax + profit_tax.result();
        if calculated_profit != net_profit.result() {
            return Err(eyre!(
                "Net profit mismatch. Calculated {} provided in report {}",
                calculated_profit.to_string(),
                net_profit.result().to_string()
            ));
        }
        Ok(profit_tax.result())
    }
}
