pub mod error;
pub mod wallet;

use lumo_common::ROOT_DATA_DIR;
use std::{path::PathBuf, sync::Arc};
use wallet::WalletsTable;

#[derive(Debug, Clone)]
pub struct Database {
    pub wallets: WalletsTable,
}

#[cfg(not(test))]
fn database_location() -> PathBuf {
    ROOT_DATA_DIR.join("lumo.db") // ~/.lumo/lumo.db
}

#[cfg(test)]
fn database_location() -> PathBuf {
    use rand::distr::Alphanumeric;
    use rand::prelude::*;

    let mut rng = rand::rng();
    let random_string: String = (0..7).map(|_| rng.sample(Alphanumeric) as char).collect();
    let lumo_db = format!("lumo_{random_string}.db");

    let test_dir = ROOT_DATA_DIR.join("test");
    std::fs::create_dir_all(&test_dir).expect("failed to create test dir");

    test_dir.join(lumo_db)
}

impl Database {
    pub fn new() -> Result<Self, error::DatabaseError> {
        Self::new_with_path(None)
    }

    pub fn new_with_path(custom_path: Option<PathBuf>) -> Result<Self, error::DatabaseError> {
        let location = custom_path.unwrap_or_else(database_location);
        let db = if location.exists() {
            redb::Database::open(&location)?
        } else {
            redb::Database::create(&location)?
        };

        let db = Arc::new(db);
        let write_txn = db.begin_write()?;
        let wallets = WalletsTable::new(db.clone(), &write_txn)?;
        write_txn.commit()?;

        Ok(Self { wallets })
    }

    #[cfg(test)]
    pub fn delete_database() {
        let _ = std::fs::remove_dir_all(ROOT_DATA_DIR.join("test"));
    }
}
