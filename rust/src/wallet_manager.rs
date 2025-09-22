use crate::wallet::error::{Result, WalletError};
use crate::wallet::{Wallet, WalletId};
use lumo_types::{Network, Transaction};
use std::collections::HashMap;

#[derive(Default)]
pub struct WalletManager {
    wallets: HashMap<WalletId, Wallet>,
    active_wallet_id: Option<WalletId>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            active_wallet_id: None,
        }
    }

    // Load or create a wallet
    pub fn add_wallet(&mut self, wallet: Wallet) -> WalletId {
        let wallet_id = wallet.id.clone();
        self.wallets.insert(wallet_id.clone(), wallet);
        if self.active_wallet_id.is_none() {
            self.active_wallet_id = Some(wallet_id.clone());
        }
        wallet_id
    }

    pub fn load_existing_wallet(&mut self, wallet_id: &WalletId, network: Network) -> Result<()> {
        let wallet = Wallet::try_load_persisted(wallet_id, network)?;
        self.wallets.insert(wallet_id.clone(), wallet);
        Ok(())
    }

    pub fn get_transactions(&self, wallet_id: &WalletId) -> Result<Vec<Transaction>> {
        self.wallets
            .get(wallet_id)
            .ok_or(WalletError::WalletNotFound(format!(
                "Wallet {wallet_id} not found"
            )))?
            .transactions()
    }

    pub fn set_active_wallet(&mut self, wallet_id: WalletId) -> Result<()> {
        if self.wallets.contains_key(&wallet_id) {
            self.active_wallet_id = Some(wallet_id);
            Ok(())
        } else {
            Err(WalletError::WalletNotFound(format!(
                "Wallet {wallet_id} not found"
            )))
        }
    }

    pub fn active_wallet(&self) -> Result<&Wallet> {
        let wallet_id = self
            .active_wallet_id
            .as_ref()
            .ok_or(WalletError::Generic("No active wallet".to_string()))?;
        self.wallets
            .get(wallet_id)
            .ok_or(WalletError::WalletNotFound(format!(
                "Active wallet {wallet_id} not found"
            )))
    }

    pub fn list_wallet_ids(&self) -> Vec<WalletId> {
        self.wallets.keys().cloned().collect()
    }
}
