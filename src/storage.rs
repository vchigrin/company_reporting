use crate::model;
use eyre::Result;
use rusqlite::{Connection, named_params};
use std::path::Path;

pub struct Storage {
    connection: Connection,
}

const COMPANIES: &str = "companies";

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
                   name TEXT NOT NULL
                );",
                    COMPANIES
                ),
                [],
            )?;
        }
        Ok(())
    }

    pub fn save_company(&self, company: &model::CompanyInfo) -> Result<()> {
        let company_id = self.insert_and_get_company_id(company)?;
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
        })
    }

    fn insert_and_get_company_id(&self, company: &model::CompanyInfo) -> Result<i64> {
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
        let res: i64 = self.connection.query_one(
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
    fn creation_in_file() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let db_path = tmp_dir.path().join("test.db");
        {
            let storage = Storage::new_with_file(&db_path).expect("Failed create DB");
            let company = model::CompanyInfo {
                name: "foo".to_owned(),
                inn: "123".to_owned(),
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
}
