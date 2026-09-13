use crate::model;
use crate::model::{BalanceKeys, GenericKeys, IncomeKeys, ParsedLineInfo};
use eyre::Result;
use rusqlite::{Connection, Transaction, named_params};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

pub struct Storage {
    connection: Connection,
}

const COMPANIES: &str = "companies";
const REPORTS: &str = "reports";
const REPORT_LINES: &str = "report_lines";
const REPORT_KEYS_DICT: &str = "report_keys_dict";

impl Storage {
    pub fn new_with_file(path: &Path) -> Result<Self> {
        let con = Connection::open(path)?;
        let result = Self { connection: con };
        result.ensure_schema()?;
        Ok(result)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let con = Connection::open_in_memory()?;
        let result = Self { connection: con };
        result.ensure_schema()?;
        Ok(result)
    }

    fn ensure_schema(&self) -> Result<()> {
        if !self.connection.table_exists(None, COMPANIES)? {
            self.connection.execute(
                &format!(
                    "CREATE TABLE {}(
                   id INTEGER PRIMARY KEY,
                   inn TEXT NOT NULL UNIQUE,
                   name TEXT NOT NULL UNIQUE
                );",
                    COMPANIES
                ),
                [],
            )?;
        }
        if !self.connection.table_exists(None, REPORTS)? {
            self.connection.execute(
                &format!(
                    "CREATE TABLE {}(
                   report_id INTEGER PRIMARY KEY,
                   company_id INTEGER NOT NULL,
                   report_type TEXT NOT NULL,
                   report_period TEXT NOT NULL,
                   CONSTRAINT unique_report UNIQUE (company_id, report_type, report_period)
                );",
                    REPORTS
                ),
                [],
            )?;
        }
        if !self.connection.table_exists(None, REPORT_LINES)? {
            self.connection.execute(
                &format!(
                    "CREATE TABLE {}(
                   report_line_id INTEGER PRIMARY KEY,
                   report_id INTEGER NOT NULL,
                   key TEXT NOT NULL,
                   amount_roubles INTEGER NOT NULL,
                   original_line TEXT,
                   line_index INTEGER NOT NULL
                );",
                    REPORT_LINES
                ),
                [],
            )?;
        }
        if !self.connection.table_exists(None, REPORT_KEYS_DICT)? {
            self.connection.execute(
                &format!(
                    "CREATE TABLE {}(
                   id INTEGER PRIMARY KEY,
                   report_type TEXT NOT NULL,
                   report_string TEXT NOT NULL,
                   key TEXT NOT NULL
                );",
                    REPORT_KEYS_DICT
                ),
                [],
            )?;
        }
        Ok(())
    }

    pub fn save_company(&mut self, company: &model::CompanyInfo) -> Result<()> {
        let tx = self.connection.transaction()?;
        let company_id = Self::insert_and_get_company_id(&tx, company)?;
        for (key, report) in &company.raw_reports {
            if let Some(lines) = &report.balance {
                let report_id = Self::insert_and_get_report_id(
                    &tx,
                    company_id,
                    key,
                    model::ReportType::Balance,
                )?;
                Self::overwrite_report_lines::<BalanceKeys>(&tx, report_id, lines)?;
            }
            if let Some(lines) = &report.income {
                let report_id = Self::insert_and_get_report_id(
                    &tx,
                    company_id,
                    key,
                    model::ReportType::Income,
                )?;
                Self::overwrite_report_lines::<IncomeKeys>(&tx, report_id, lines)?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_company_by_inn(&self, inn: &str) -> Result<model::CompanyInfo> {
        let (company_id, name): (i64, String) = self.connection.query_row(
            &format!("SELECT id, name FROM {} WHERE inn=?", COMPANIES),
            [inn],
            |r| Ok((r.get_unwrap(0), r.get_unwrap(1))),
        )?;
        Ok(model::CompanyInfo {
            name,
            inn: inn.to_owned(),
            raw_reports: self.load_reports(company_id)?,
        })
    }

    pub fn get_company_by_name(&self, name: &str) -> Result<model::CompanyInfo> {
        let (company_id, inn): (i64, String) = self.connection.query_row(
            &format!("SELECT id, inn FROM {} WHERE name=?", COMPANIES),
            [name],
            |r| Ok((r.get_unwrap(0), r.get_unwrap(1))),
        )?;
        Ok(model::CompanyInfo {
            name: name.to_owned(),
            inn,
            raw_reports: self.load_reports(company_id)?,
        })
    }

    pub fn load_keys_dict<Keys: GenericKeys>(
        &self,
        report_type: model::ReportType,
    ) -> Result<HashMap<String, Keys>> {
        let report_type_str: &'static str = report_type.into();
        let mut stmt = self
            .connection
            .prepare(&format!(
                "SELECT report_string, key FROM {} WHERE report_type = $report_type",
                REPORT_KEYS_DICT
            ))
            .unwrap();
        let mut rows = stmt
            .query(named_params! {"$report_type": report_type_str})
            .unwrap();
        let mut result = HashMap::new();
        while let Some(row) = rows.next().unwrap() {
            let report_string = row.get::<usize, String>(0)?;
            let key_str = row.get::<usize, String>(1)?;
            let key = Keys::from_str(&key_str)?;
            result.insert(report_string, key);
        }
        Ok(result)
    }

    pub fn overwrite_keys_dict<Keys: GenericKeys>(
        &mut self,
        report_type: model::ReportType,
        dict: &HashMap<String, Keys>,
    ) -> Result<()> {
        let report_type_str: &'static str = report_type.into();
        let tx = self.connection.transaction()?;
        tx.execute(
            &format!(
                "DELETE FROM {} WHERE report_type = $report_type",
                REPORT_KEYS_DICT
            ),
            named_params! {"$report_type": report_type_str},
        )?;
        let mut insert_stmt = tx
            .prepare(&format!(
                "INSERT INTO {} (report_type, report_string, key) VALUES(
                   $report_type,
                   $report_string,
                   $key
                )",
                REPORT_KEYS_DICT
            ))
            .unwrap();
        for (report_string, key) in dict {
            let key_str: &'static str = (*key).into();
            insert_stmt.execute(named_params! {
               "$report_type": report_type_str,
               "$report_string": report_string,
               "$key": key_str,
            })?;
        }
        drop(insert_stmt);
        tx.commit()?;
        Ok(())
    }

    pub fn list_companies(&self) -> Result<Vec<model::CompanyInfo>> {
        let mut stmt = self
            .connection
            .prepare(&format!("SELECT id, inn, name FROM {}", COMPANIES))
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut result = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let company_id = row.get::<usize, i64>(0)?;
            let inn = row.get::<usize, String>(1)?;
            let name = row.get::<usize, String>(2)?;
            result.push(model::CompanyInfo {
                name,
                inn,
                raw_reports: self.load_reports(company_id)?,
            });
        }
        Ok(result)
    }

    fn overwrite_report_lines<'a, Key: GenericKeys>(
        tx: &Transaction<'a>,
        report_id: i64,
        lines: &[ParsedLineInfo<Key>],
    ) -> Result<()> {
        let drop_query = format!("DELETE FROM {} WHERE report_id = $report_id;", REPORT_LINES);
        tx.execute(
            &drop_query,
            named_params! {
               "$report_id": report_id,
            },
        )?;
        let mut insert_stmt = tx
            .prepare(&format!(
                "INSERT INTO {} (report_id, key, amount_roubles, original_line, line_index) VALUES(
                   $report_id,
                   $key,
                   $amount_roubles,
                   $original_line,
                   $line_index
                )",
                REPORT_LINES
            ))
            .unwrap();
        for (index, line) in lines.iter().enumerate() {
            let key_str: &'static str = line.key.into();
            insert_stmt.execute(named_params! {
               "$report_id": report_id,
               "$line_index": index as i64,
               "$key": key_str,
               "$amount_roubles": line.value.in_roubles(),
               "$original_line": line.original_line,
            })?;
        }
        Ok(())
    }

    fn load_reports(&self, company_id: i64) -> Result<HashMap<model::Period, model::RawReport>> {
        let mut stmt = self.connection.prepare(&format!(
            "SELECT report_id, report_type, report_period FROM {} WHERE company_id = $company_id",
            REPORTS
        )).unwrap();
        let mut rows = stmt
            .query(named_params! {"$company_id": company_id})
            .unwrap();
        let mut result = HashMap::<model::Period, model::RawReport>::new();
        while let Some(row) = rows.next().unwrap() {
            let report_id = row.get::<usize, i64>(0)?;
            let report_type_str = row.get::<usize, String>(1)?;
            let report_period_str = row.get::<usize, String>(2)?;
            let report_type = model::ReportType::from_str(&report_type_str)?;

            let report_period = model::Period::from_short_string(&report_period_str)?;
            if let Some(r) = result.get_mut(&report_period) {
                self.fill_raw_report(r, report_type, report_id)?;
            } else {
                let mut new_report = model::RawReport {
                    balance: None,
                    income: None,
                };
                self.fill_raw_report(&mut new_report, report_type, report_id)?;
                result.insert(report_period, new_report);
            }
        }
        Ok(result)
    }

    fn fill_raw_report(
        &self,
        r: &mut model::RawReport,
        report_type: model::ReportType,
        report_id: i64,
    ) -> Result<()> {
        match report_type {
            model::ReportType::Balance => {
                assert!(r.balance.is_none());
                let lines = self.load_report_lines::<BalanceKeys>(report_id)?;
                r.balance = Some(lines);
            }
            model::ReportType::Income => {
                assert!(r.income.is_none());
                let lines = self.load_report_lines::<IncomeKeys>(report_id)?;
                r.income = Some(lines);
            }
        }
        Ok(())
    }

    fn load_report_lines<Keys: GenericKeys>(
        &self,
        report_id: i64,
    ) -> Result<Vec<ParsedLineInfo<Keys>>> {
        let mut stmt = self
            .connection
            .prepare(&format!(
                "SELECT key, amount_roubles, original_line FROM {} WHERE report_id = $report_id
                 ORDER BY line_index",
                REPORT_LINES
            ))
            .unwrap();
        let mut rows = stmt.query(named_params! {"$report_id": report_id}).unwrap();
        let mut result = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let key_str = row.get::<usize, String>(0)?;
            let amount_roubles = row.get::<usize, i64>(1)?;
            let original_line = row.get::<usize, String>(2)?;
            let key = Keys::from_str(&key_str)?;
            result.push(ParsedLineInfo::<Keys> {
                key,
                value: model::Money::from_roubles(amount_roubles),
                original_line,
            });
        }
        Ok(result)
    }

    fn insert_and_get_report_id<'a>(
        tx: &Transaction<'a>,
        company_id: i64,
        period: &model::Period,
        report_type: model::ReportType,
    ) -> Result<i64> {
        let query = format!(
            "INSERT OR REPLACE INTO {} (company_id, report_type, report_period)
             VALUES($company_id, $report_type, $report_period)
             RETURNING report_id;",
            REPORTS
        );
        let report_type_str: &'static str = report_type.into();
        let res: i64 = tx.query_one(
            &query,
            named_params! {
               "$company_id": company_id,
               "$report_type": report_type_str,
               "$report_period": period.short_string(),
            },
            |r| r.get(0),
        )?;
        Ok(res)
    }

    fn insert_and_get_company_id(tx: &Transaction, company: &model::CompanyInfo) -> Result<i64> {
        let query = format!(
            "INSERT INTO {} (inn, name)
             VALUES($inn, $name)
             ON CONFLICT
             DO UPDATE SET
               inn = $inn,
               name = $name
             RETURNING id;",
            COMPANIES
        );
        let res: i64 = tx.query_one(
            &query,
            named_params! {
               "$inn": company.inn,
               "$name": company.name,
            },
            |r| r.get(0),
        )?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_in_memory() {
        Storage::new_in_memory().expect("Failed create DB");
    }

    #[test]
    fn load_keys_dict() {
        let storage = Storage::new_in_memory().expect("Failed create DB");
        let balance_dict = storage
            .load_keys_dict::<BalanceKeys>(model::ReportType::Balance)
            .expect("Failed load balance dict");
        let income_dict = storage
            .load_keys_dict::<IncomeKeys>(model::ReportType::Income)
            .expect("Failed load income dict");
        // Empty DB must yield empty dicts.
        assert!(balance_dict.is_empty());
        assert!(income_dict.is_empty());
    }

    #[test]
    fn overwrite_keys_dict() {
        let mut storage = Storage::new_in_memory().expect("Failed create DB");

        let mut balance_dict = HashMap::new();
        balance_dict.insert("основные средства".to_owned(), BalanceKeys::FixedAssets);
        balance_dict.insert("денежные средства".to_owned(), BalanceKeys::Cash);

        let mut income_dict = HashMap::new();
        income_dict.insert("выручка от реализации".to_owned(), IncomeKeys::SalesRevenue);
        income_dict.insert("чистая прибыль".to_owned(), IncomeKeys::NetProfit);

        storage
            .overwrite_keys_dict(model::ReportType::Balance, &balance_dict)
            .expect("Failed overwrite balance dict");
        storage
            .overwrite_keys_dict(model::ReportType::Income, &income_dict)
            .expect("Failed overwrite income dict");

        let loaded_balance = storage
            .load_keys_dict::<BalanceKeys>(model::ReportType::Balance)
            .expect("Failed load balance dict");
        assert_eq!(loaded_balance, balance_dict);

        let loaded_income = storage
            .load_keys_dict::<IncomeKeys>(model::ReportType::Income)
            .expect("Failed load income dict");
        assert_eq!(loaded_income, income_dict);

        // Overwriting balance must replace previous rows and leave income intact.
        let mut new_balance_dict = HashMap::new();
        new_balance_dict.insert("запасы".to_owned(), BalanceKeys::PhysicalInventory);
        storage
            .overwrite_keys_dict(model::ReportType::Balance, &new_balance_dict)
            .expect("Failed overwrite balance dict again");
        let loaded_balance = storage
            .load_keys_dict::<BalanceKeys>(model::ReportType::Balance)
            .expect("Failed load balance dict");
        assert_eq!(loaded_balance, new_balance_dict);
        let loaded_income = storage
            .load_keys_dict::<IncomeKeys>(model::ReportType::Income)
            .expect("Failed load income dict");
        assert_eq!(loaded_income, income_dict);
    }

    #[test]
    fn creation_in_file() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let db_path = tmp_dir.path().join("test.db");
        {
            let mut storage = Storage::new_with_file(&db_path).expect("Failed create DB");
            let company = model::CompanyInfo {
                name: "foo".to_owned(),
                inn: "123".to_owned(),
                raw_reports: HashMap::default(),
            };
            storage
                .save_company(&company)
                .expect("Failed insert company");
        }
        {
            // Ensure that creation second time will open, not overwrite DB
            let storage = Storage::new_with_file(&db_path).expect("Failed create DB");
            let company = storage
                .get_company_by_inn("123")
                .expect("Failed query company");
            assert_eq!(company.name, "foo");
        }
    }

    fn sample_company() -> model::CompanyInfo {
        let mut raw_reports = HashMap::new();
        for (period, balance, income) in [
            (
                model::Period::full(2023),
                vec![
                    ParsedLineInfo {
                        key: BalanceKeys::FixedAssets,
                        value: model::Money::from_thousands(1500),
                        original_line: "Основные средства 1500".to_owned(),
                    },
                    ParsedLineInfo {
                        key: BalanceKeys::Cash,
                        value: model::Money::from_roubles(12345),
                        original_line: "Денежные средства 12345".to_owned(),
                    },
                ],
                vec![ParsedLineInfo {
                    key: IncomeKeys::SalesRevenue,
                    value: model::Money::from_thousands(9000),
                    original_line: "Выручка от реализации 9000".to_owned(),
                }],
            ),
            (
                model::Period::first_half(2024),
                vec![ParsedLineInfo {
                    key: BalanceKeys::AccountsPayable,
                    value: model::Money::from_millions(2),
                    original_line: "Кредиторская задолженность 2".to_owned(),
                }],
                vec![ParsedLineInfo {
                    key: IncomeKeys::NetProfit,
                    value: model::Money::from_roubles(-500),
                    original_line: "Чистая прибыль -500".to_owned(),
                }],
            ),
        ] {
            raw_reports.insert(
                period,
                model::RawReport {
                    balance: Some(balance),
                    income: Some(income),
                },
            );
        }

        model::CompanyInfo {
            name: "ООО Пример".to_owned(),
            inn: "7701234567".to_owned(),
            raw_reports,
        }
    }

    #[test]
    fn raw_reports_save_restore() {
        let mut storage = Storage::new_in_memory().expect("Failed create DB");
        let original = sample_company();
        storage
            .save_company(&original)
            .expect("Failed save company");
        let restored = storage
            .get_company_by_inn(&original.inn)
            .expect("Failed query company");

        assert_eq!(restored.name, original.name);
        assert_eq!(restored.inn, original.inn);
        assert_eq!(restored.raw_reports.len(), original.raw_reports.len());
        for (period, raw_report) in &original.raw_reports {
            let restored_report = restored
                .raw_reports
                .get(period)
                .expect("Missing restored period");
            compare_raw_report(restored_report, raw_report);
        }
    }

    fn compare_raw_report(restored: &model::RawReport, expected: &model::RawReport) {
        let (restored_balance, expected_balance) =
            (restored.balance.as_ref(), expected.balance.as_ref());
        match (restored_balance, expected_balance) {
            (Some(rb), Some(eb)) => compare_lines(rb, eb),
            (None, None) => {}
            _ => panic!("balance presence mismatch"),
        }

        let (restored_income, expected_income) =
            (restored.income.as_ref(), expected.income.as_ref());
        match (restored_income, expected_income) {
            (Some(ri), Some(ei)) => compare_lines(ri, ei),
            (None, None) => {}
            _ => panic!("income presence mismatch"),
        }
    }

    fn compare_lines<Keys: crate::model::GenericKeys>(
        restored: &[crate::model::ParsedLineInfo<Keys>],
        expected: &[crate::model::ParsedLineInfo<Keys>],
    ) {
        assert_eq!(restored.len(), expected.len());
        for (r, e) in restored.iter().zip(expected.iter()) {
            assert_eq!(r.key, e.key);
            assert_eq!(r.value, e.value);
            assert_eq!(r.original_line, e.original_line);
        }
    }

    #[test]
    fn overwrite_reports() {
        fn balance_line(key: BalanceKeys, roubles: i64, line: &str) -> ParsedLineInfo<BalanceKeys> {
            ParsedLineInfo {
                key,
                value: model::Money::from_roubles(roubles),
                original_line: line.to_owned(),
            }
        }
        fn balance_only_report(lines: Vec<ParsedLineInfo<BalanceKeys>>) -> model::RawReport {
            model::RawReport {
                balance: Some(lines),
                income: None,
            }
        }

        let mut company = model::CompanyInfo {
            name: "ООО Пример".to_owned(),
            inn: "7707654321".to_owned(),
            raw_reports: HashMap::new(),
        };
        macro_rules! insert_report {
            ($period:expr, $lines:expr) => {
                company
                    .raw_reports
                    .insert($period, balance_only_report($lines));
            };
        }

        let period_add = model::Period::full(2023);
        let period_remove = model::Period::full(2024);
        let period_change = model::Period::full(2025);

        // Initial state: 2 lines each.
        insert_report!(
            period_add,
            vec![
                balance_line(BalanceKeys::FixedAssets, 1000, "Основные средства 1000"),
                balance_line(BalanceKeys::Cash, 200, "Денежные средства 200"),
            ]
        );
        insert_report!(
            period_remove,
            vec![
                balance_line(BalanceKeys::FixedAssets, 1000, "Основные средства 1000"),
                balance_line(BalanceKeys::Cash, 200, "Денежные средства 200"),
                balance_line(BalanceKeys::AccountsReceivable, 300, "Дебиторка 300"),
            ]
        );
        insert_report!(
            period_change,
            vec![
                balance_line(BalanceKeys::FixedAssets, 1000, "Основные средства 1000"),
                balance_line(BalanceKeys::Cash, 200, "Денежные средства 200"),
            ]
        );

        let mut storage = Storage::new_in_memory().expect("Failed create DB");
        storage.save_company(&company).expect("Failed save company");

        // Modify in memory: add a line to one report, remove from another,
        // change a line in the third.
        let modified = company.raw_reports.get_mut(&period_add).unwrap();
        modified.balance.as_mut().unwrap().push(balance_line(
            BalanceKeys::AccountsReceivable,
            300,
            "Дебиторка 300",
        ));

        company
            .raw_reports
            .get_mut(&period_remove)
            .unwrap()
            .balance
            .as_mut()
            .unwrap()
            .remove(2);

        {
            let lines = company
                .raw_reports
                .get_mut(&period_change)
                .unwrap()
                .balance
                .as_mut()
                .unwrap();
            lines[1] = balance_line(BalanceKeys::Cash, 999, "Денежные средства 999");
        }

        storage
            .save_company(&company)
            .expect("Failed overwrite company");

        let restored = storage
            .get_company_by_inn(&company.inn)
            .expect("Failed query company");
        assert_eq!(restored.raw_reports.len(), company.raw_reports.len());
        for (period, expected_report) in &company.raw_reports {
            let restored_report = restored
                .raw_reports
                .get(period)
                .expect("Missing restored period");
            compare_raw_report(restored_report, expected_report);
        }
    }
}
