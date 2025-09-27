use crate::database::error::DatabaseError;
use crate::wallet::metadata::{WalletId, WalletMetadata};
use lumo_types::Network;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use serde_json;
use std::sync::Arc;

const TABLE: TableDefinition<&'static str, &'static str> = TableDefinition::new("wallet_metadata");

#[derive(Debug, Clone)]
pub struct WalletsTable {
    db: Arc<redb::Database>,
}

impl WalletsTable {
    pub fn new(
        db: Arc<redb::Database>,
        write_txn: &redb::WriteTransaction,
    ) -> Result<Self, DatabaseError> {
        let _table = write_txn.open_table(TABLE)?;

        Ok(Self { db })
    }
    pub fn save_new_wallet_metadata(&self, wallet: WalletMetadata) -> Result<(), DatabaseError> {
        let wallet_json = serde_json::to_string(&wallet)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(wallet.id.to_string().as_str(), wallet_json.as_str())?;
        }
        write_txn.commit()?;

        Ok(())
    }
    pub fn get(&self, id: &WalletId) -> Result<Option<WalletMetadata>, DatabaseError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        match table.get(id.to_string().as_str())? {
            Some(json_data) => {
                let wallet: WalletMetadata = serde_json::from_str(json_data.value())?;
                Ok(Some(wallet))
            }
            None => Ok(None),
        }
    }

    pub fn get_all(&self, network: Option<Network>) -> Result<Vec<WalletMetadata>, DatabaseError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let mut wallets = Vec::new();

        for item in table.iter()? {
            let (_id, json_data) = item?;
            let wallet: WalletMetadata = serde_json::from_str(json_data.value())?;
            if let Some(filter_network) = network {
                if wallet.network == filter_network {
                    wallets.push(wallet);
                }
            } else {
                wallets.push(wallet);
            }
        }

        Ok(wallets)
    }

    #[cfg(test)]
    pub fn clear_all(&self) -> Result<(), DatabaseError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            // Get all keys first to avoid iterator invalidation
            let keys: Result<Vec<String>, DatabaseError> = table
                .iter()?
                .map(|item| {
                    let (key, _) = item.map_err(DatabaseError::from)?;
                    Ok(key.value().to_string())
                })
                .collect();

            // Remove all entries
            for key in keys? {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
