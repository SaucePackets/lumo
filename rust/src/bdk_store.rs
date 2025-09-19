use std::path::PathBuf;

use lumo_common::consts::ROOT_DATA_DIR;
use crate::wallet::WalletId;
use lumo_types::Network;
use eyre::{Context, Result};

#[allow(dead_code)]
pub struct BDKStore {
    id: WalletId,
    network: Network,
    pub conn: bdk_wallet::rusqlite::Connection,
}

fn sqlite_data_path(wallet_id: &WalletId) -> PathBuf {
    let db = format!("bdk_wallet_sqlite_{}.db", wallet_id.to_string().to_lowercase());
    ROOT_DATA_DIR.join(db)
}

impl BDKStore {
    pub fn try_new(id: &WalletId, network: impl Into<Network>) -> Result<Self> {
        // Create database file path
        let sqlite_data_path = sqlite_data_path(id);

        // Open connection to the database
        let conn = bdk_wallet::rusqlite::Connection::open(&sqlite_data_path)
            .context("unable to open rusqlite connection")?;

        Ok(Self {
            id: id.clone(),
            network: network.into(),
            conn,
        })
    }
}
