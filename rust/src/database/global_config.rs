use crate::database::error::DatabaseError;
use crate::wallet::WalletId;
use redb::{ReadableDatabase, TableDefinition};
use std::sync::Arc;

const TABLE: TableDefinition<&'static str, &'static str> = TableDefinition::new("global_config");

#[derive(Debug, Clone)]
pub struct GlobalConfigTable {
    db: Arc<redb::Database>,
}

impl GlobalConfigTable {
    pub fn new(
        db: Arc<redb::Database>,
        write_txn: &redb::WriteTransaction,
    ) -> Result<Self, DatabaseError> {
        let _table = write_txn.open_table(TABLE)?;
        Ok(Self { db })
    }

    // Set the selected wallet id
    pub fn select_wallet(&self, wallet_id: &WalletId) -> Result<(), DatabaseError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert("selected_wallet_id", wallet_id.to_string().as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // Get the currently selected wallet id
    pub fn selected_wallet(&self) -> Result<Option<WalletId>, DatabaseError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        match table.get("selected_wallet_id")? {
            Some(wallet_id_str) => {
                let wallet_id = WalletId::from_string(wallet_id_str.value())?;
                Ok(Some(wallet_id))
            }
            None => Ok(None),
        }
    }

    pub fn clear_selected_wallet(&self) -> Result<(), DatabaseError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.remove("selected_wallet_id")?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
