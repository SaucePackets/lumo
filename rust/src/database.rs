pub mod error;
pub mod global_config;
pub mod wallet;

use arc_swap::ArcSwap;
use global_config::GlobalConfigTable;
use lumo_common::ROOT_DATA_DIR;
use once_cell::sync::OnceCell;
use std::{path::PathBuf, sync::Arc};
use wallet::WalletsTable;

pub static DATABASE: OnceCell<ArcSwap<Database>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct Database {
    pub wallets: WalletsTable,
    pub global_config: GlobalConfigTable,
}

#[cfg(not(test))]
fn database_location() -> PathBuf {
    ROOT_DATA_DIR.join("lumo.db") // ~/.lumo/lumo.db
}

#[cfg(test)]
fn database_location() -> PathBuf {
    // Use a single test database for all tests
    let test_dir = ROOT_DATA_DIR.join("test");
    std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    test_dir.join("lumo_test.db")
}

impl Database {
    pub fn global() -> Arc<Self> {
        let db = DATABASE
            .get_or_init(|| ArcSwap::new(Arc::new(Self::init())))
            .load();
        Arc::clone(&db)
    }

    fn init() -> Database {
        let location = database_location();
        let db = if location.exists() {
            redb::Database::open(&location).expect("failed to open database")
        } else {
            redb::Database::create(&location).expect("failed to create database")
        };

        let db = Arc::new(db);
        let write_txn = db.begin_write().expect("failed to begin write transaction");
        let wallets =
            WalletsTable::new(db.clone(), &write_txn).expect("failed to create wallets table");

        let global_config = GlobalConfigTable::new(db.clone(), &write_txn)
            .expect("failed to create global config table");

        write_txn
            .commit()
            .expect("failed to commit write transaction");

        Database {
            wallets,
            global_config,
        }
    }

    #[cfg(test)]
    pub fn delete_database() {
        // Clear wallet data from the current database instance
        if let Some(arc_swap) = DATABASE.get() {
            let db = arc_swap.load();
            let _ = db.wallets.clear_all();
        }

        // Also remove the test directory for cleanup
        let _ = std::fs::remove_dir_all(ROOT_DATA_DIR.join("test"));
    }
}
