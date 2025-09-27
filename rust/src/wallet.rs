pub mod balance;
pub mod error;
pub mod metadata;
pub use metadata::{WalletId, WalletMetadata, WalletType};

#[cfg(test)]
mod dev_tests;

use crate::GAP_LIMIT;
use bdk_wallet::{
    chain::ChainPosition as BdkChainPosition,
    template::{Bip84, DescriptorTemplate},
    KeychainKind, Wallet as BdkWallet,
};
use bip39::Mnemonic;
use rand::Rng;
use std::path::PathBuf;
use std::str::FromStr;

use crate::bdk_store::BDKStore;
use crate::database::Database;
use crate::node::client::esplora::EsploraClient;
use crate::node::Node;
use crate::wallet::balance::Balance;
use crate::wallet::error::{Result, WalletError};
use lumo_types::address::AddressInfo;
use lumo_types::{
    transaction::{ConfirmationStatus, TransactionDirection, TransactionId},
    Address, Amount as LumoAmount, Network, Transaction,
};

type PersistedBdkWallet = bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>;

/// Lumo Bitcoin wallet
#[derive(Debug)]
pub struct Wallet {
    pub id: WalletId,
    pub metadata: WalletMetadata,
    pub bdk: bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>,
}

impl Wallet {
    /// Create a new wallet from mnemonic phrase
    pub fn new_from_mnemonic(
        name: String,
        mnemonic_phrase: &str,
        network: Network,
    ) -> Result<Self> {
        // Parse and validate mnemonic
        let mnemonic = Mnemonic::from_str(mnemonic_phrase)?;

        // Create metadata
        let metadata = WalletMetadata::new(name, network);

        // Create BDK wallet with Native SegWit (bech32)
        let bdk_wallet = Self::create_bdk_wallet(&mnemonic, network, &metadata.id, None)?;

        // Save metadata to database
        let database = Database::new()?;
        database
            .wallets
            .save_new_wallet_metadata(metadata.clone())?;

        Ok(Self {
            id: metadata.id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    /// Create a new wallet from mnemonic phrase with custom database path
    #[cfg(test)]
    pub fn new_from_mnemonic_with_db_path(
        name: String,
        mnemonic_phrase: &str,
        network: Network,
        db_path: PathBuf,
    ) -> Result<Self> {
        // Parse and validate mnemonic
        let mnemonic = Mnemonic::from_str(mnemonic_phrase)?;

        // Create metadata
        let metadata = WalletMetadata::new(name, network);

        // Create BDK wallet with Native SegWit (bech32)
        let bdk_wallet = Self::create_bdk_wallet(&mnemonic, network, &metadata.id, None)?;

        // Save metadata to database
        let database = Database::new_with_path(Some(db_path))?;
        database
            .wallets
            .save_new_wallet_metadata(metadata.clone())?;

        Ok(Self {
            id: metadata.id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    /// Create a new wallet with random mnemonic
    pub fn new_random(name: String, network: Network) -> Result<(Self, Mnemonic)> {
        // Generate random mnemonic (12 words = 128 bits = 16 bytes)
        let random_bytes = rand::rng().random::<[u8; 16]>();
        let mnemonic =
            Mnemonic::from_entropy(&random_bytes).map_err(WalletError::InvalidMnemonic)?;

        // Create metadata
        let metadata = WalletMetadata::new(name, network);

        // Create BDK wallet
        let bdk_wallet = Self::create_bdk_wallet(&mnemonic, network, &metadata.id, None)?;

        // Save metadata to database
        let database = Database::new()?;
        database
            .wallets
            .save_new_wallet_metadata(metadata.clone())?;

        let wallet = Self {
            id: metadata.id.clone(),
            metadata,
            bdk: bdk_wallet,
        };

        Ok((wallet, mnemonic))
    }

    /// Create BDK wallet from mnemonic using BIP84 (Native SegWit)
    fn create_bdk_wallet(
        mnemonic: &Mnemonic,
        network: Network,
        wallet_id: &WalletId,
        passphrase: Option<&str>,
    ) -> Result<PersistedBdkWallet> {
        // Convert our Network to BDK's network
        let bdk_network = network.to_bitcoin_network();

        // Create seed from mnemonic
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));

        // Derive the master extended private key
        let xpriv = bitcoin::bip32::Xpriv::new_master(bdk_network, &seed)
            .map_err(|e| WalletError::Bitcoin(e.to_string()))?;

        // Use BDK's BIP84 template to create descriptors (Native SegWit)
        let (external_descriptor, external_keymap, _) = Bip84(xpriv, KeychainKind::External)
            .build(bdk_network)
            .map_err(|e| WalletError::Bdk(e.to_string()))?;

        let (internal_descriptor, internal_keymap, _) = Bip84(xpriv, KeychainKind::Internal)
            .build(bdk_network)
            .map_err(|e| WalletError::Bdk(e.to_string()))?;

        let mut store = BDKStore::try_new(wallet_id, network)?;

        // Create BDK wallet (in-memory for now, no persistence)
        let wallet = BdkWallet::create(
            (external_descriptor, external_keymap),
            (internal_descriptor, internal_keymap),
        )
        .network(bdk_network)
        .create_wallet(&mut store.conn)
        .map_err(|e| WalletError::Bdk(e.to_string()))?;

        Ok(wallet)
    }

    pub fn try_load_persisted(wallet_id: &WalletId, network: Network) -> Result<Self> {
        let mut store = BDKStore::try_new(wallet_id, network)?;

        let bdk_wallet = bdk_wallet::Wallet::load()
            .load_wallet(&mut store.conn)
            .map_err(|e| WalletError::Bdk(e.to_string()))?
            .ok_or(WalletError::WalletNotFound("Wallet not found".to_string()))?;

        let database = Database::new()?;

        let metadata = match database.wallets.get(wallet_id)? {
            Some(metadata) => metadata,
            None => WalletMetadata::new(format!("Loaded Wallet {wallet_id}"), network),
        };

        Ok(Self {
            id: wallet_id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    /// Load a persisted wallet with custom database path
    #[cfg(test)]
    pub fn try_load_persisted_with_db_path(
        wallet_id: &WalletId,
        network: Network,
        db_path: PathBuf,
    ) -> Result<Self> {
        let mut store = BDKStore::try_new(wallet_id, network)?;

        let bdk_wallet = bdk_wallet::Wallet::load()
            .load_wallet(&mut store.conn)
            .map_err(|e| WalletError::Bdk(e.to_string()))?
            .ok_or(WalletError::WalletNotFound("Wallet not found".to_string()))?;

        let database = Database::new_with_path(Some(db_path))?;

        let metadata = match database.wallets.get(wallet_id)? {
            Some(metadata) => metadata,
            None => WalletMetadata::new(format!("Loaded Wallet {wallet_id}"), network),
        };

        Ok(Self {
            id: wallet_id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>> {
        let transactions = self
            .bdk
            .transactions()
            .map(|canonical_tx| {
                let (sent, received) = self.bdk.sent_and_received(&canonical_tx.tx_node.tx);
                let direction = if sent.to_sat() > received.to_sat() {
                    TransactionDirection::Outgoing
                } else {
                    TransactionDirection::Incoming
                };

                let confirmation_status = match canonical_tx.chain_position {
                    BdkChainPosition::Unconfirmed { .. } => ConfirmationStatus::Unconfirmed,
                    BdkChainPosition::Confirmed {
                        anchor: block_time, ..
                    } => ConfirmationStatus::Confirmed {
                        block_height: block_time.block_id.height,
                    },
                };

                let txid = TransactionId::from(canonical_tx.tx_node.tx.compute_txid());
                let amount = if direction == TransactionDirection::Incoming {
                    LumoAmount::from(received)
                } else {
                    LumoAmount::from(sent)
                };

                Transaction::new(txid, amount, direction, confirmation_status, None)
            })
            .collect();

        Ok(transactions)
    }

    pub fn balance(&self) -> Balance {
        Balance(self.bdk.balance())
    }

    pub async fn sync(&mut self) -> Result<()> {
        let node = Node::default(self.network());
        let esplora_client = EsploraClient::new(&node.url).await?;
        let scan_request = self.bdk.start_full_scan().build();
        let scan_result = esplora_client
            .full_scan(scan_request, GAP_LIMIT as usize)
            .await?;
        self.bdk
            .apply_update(scan_result)
            .map_err(|e| WalletError::Generic(e.to_string()))?;
        Ok(())
    }

    /// Get a new receiving address with gap limit protection
    pub fn get_new_address(&mut self) -> Result<Address> {
        const MAX_ADDRESSES: usize = (GAP_LIMIT - 5) as usize; // 25 addresses max

        // Get unused addresses to check how many we have
        let unused_addresses: Vec<_> = self
            .bdk
            .list_unused_addresses(KeychainKind::External)
            .take(MAX_ADDRESSES)
            .collect();

        // If we have fewer than 25 revealed addresses, reveal a new one
        if unused_addresses.len() < MAX_ADDRESSES {
            let address_info = self.bdk.reveal_next_address(KeychainKind::External);
            let address = Address::new(address_info.address);
            return Ok(address);
        }

        // If we already have 25 addresses, cycle through unused ones
        if let Some(first_unused) = unused_addresses.first() {
            let address = Address::new(first_unused.address.clone());
            return Ok(address);
        }

        // Fallback: reveal next address anyway (shouldn't happen in normal usage)
        let address_info = self.bdk.reveal_next_address(KeychainKind::External);
        let address = Address::new(address_info.address);
        Ok(address)
    }

    /// Get current receiving address (doesn't increment)
    pub fn get_current_address(&self) -> Result<Address> {
        let address_info = self.bdk.peek_address(KeychainKind::External, 0);
        let address = Address::new(address_info.address);
        Ok(address)
    }

    /// Get address at specific index
    pub fn address_at(&self, index: u32) -> Result<Address> {
        let address_info = self.bdk.peek_address(KeychainKind::External, index);
        let address = Address::new(address_info.address);
        Ok(address)
    }

    /// Get first address (index 0)
    pub fn first_address(&self) -> Result<Address> {
        self.address_at(0)
    }

    pub fn build_transaction(
        &mut self,
        recipient: Address,
        amount: LumoAmount,
        fee_rate: impl Into<bitcoin::FeeRate>,
    ) -> Result<bitcoin::psbt::Psbt> {
        let mut tx_builder = self.bdk.build_tx();

        tx_builder.add_recipient(
            recipient.to_bdk_address().script_pubkey(),
            bitcoin::Amount::from_sat(amount.as_sat()),
        );
        tx_builder.fee_rate(fee_rate.into());

        let psbt = tx_builder
            .finish()
            .map_err(|e| WalletError::Generic(format!("Error building transaction: {e}")))?;

        Ok(psbt)
    }

    pub fn sign_transaction(
        &mut self,
        mut psbt: bitcoin::psbt::Psbt,
    ) -> Result<bitcoin::Transaction> {
        use bdk_wallet::SignOptions;

        let finalized = self
            .bdk
            .sign(&mut psbt, SignOptions::default())
            .map_err(|e| {
                WalletError::Generic(format!("Error signing transaction: {}", e.to_string()))
            })?;

        if !finalized {
            return Err(WalletError::Generic(
                "Transaction could not be finalized - see debug output above".to_string(),
            ));
        }

        let tx = psbt.extract_tx().map_err(|e| {
            WalletError::Generic(format!("Error extracting transaction: {}", e.to_string()))
        })?;

        Ok(tx)
    }

    pub async fn broadcast_transaction(&mut self, transaction: bitcoin::Transaction) -> Result<()> {
        let node = Node::default(self.network());
        let esplora_client = EsploraClient::new(&node.url).await?;

        esplora_client
            .broadcast_transaction(&transaction)
            .await
            .map_err(|e| {
                WalletError::Generic(format!("Error broadcasting transaction: {}", e.to_string()))
            })?;

        Ok(())
    }

    pub fn get_all_addresses(&self) -> Result<Vec<AddressInfo>> {
        let mut addresses = Vec::new();

        // Get unused addresses to find the highest revealed index
        let unused_addresses: Vec<_> = self
            .bdk
            .list_unused_addresses(KeychainKind::External)
            .collect();

        if unused_addresses.is_empty() {
            // No unused addresses - either no addresses revealed yet, or all are used
            // For a new wallet, just return the first address
            let address_info = self.bdk.peek_address(KeychainKind::External, 0);
            let address = Address::new(address_info.address.clone());
            let is_used = self.is_address_used(&address)?;
            let balance = LumoAmount::ZERO;

            addresses.push(AddressInfo {
                address: address_info.address.to_string(),
                index: address_info.index,
                is_used,
                balance,
            });
        } else {
            // Find the highest index among unused addresses
            let max_revealed_index = unused_addresses.iter().map(|a| a.index).max().unwrap_or(0);

            // Get all addresses from 0 to max_revealed_index
            for i in 0..=max_revealed_index {
                let address_info = self.bdk.peek_address(KeychainKind::External, i);
                let address = Address::new(address_info.address.clone());
                let is_used = self.is_address_used(&address)?;
                let balance = LumoAmount::ZERO;

                addresses.push(AddressInfo {
                    address: address_info.address.to_string(),
                    index: address_info.index,
                    is_used,
                    balance,
                });
            }
        }

        Ok(addresses)
    }

    pub fn is_address_used(&self, address: &Address) -> Result<bool> {
        // Check if address has received any funds by looking at transactions
        let balance = self.bdk.balance();

        // If wallet has no transactions, no addresses are used
        if balance.total().to_sat() == 0 {
            return Ok(false);
        }

        // For wallets with transactions, check if this specific address has been used
        // by checking if it appears in unused addresses list
        let unused_addresses: Vec<_> = self
            .bdk
            .list_unused_addresses(KeychainKind::External)
            .collect();

        for addr_info in unused_addresses {
            if addr_info.address.to_string() == address.as_str() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Get wallet network
    pub fn network(&self) -> Network {
        self.metadata.network
    }

    /// Get wallet name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[test]
    fn test_wallet_creation_random() {
        Database::delete_database();
        let (wallet, mnemonic) =
            Wallet::new_random("Test Wallet".to_string(), Network::Regtest).unwrap();

        assert_eq!(wallet.name(), "Test Wallet");
        assert_eq!(wallet.network(), Network::Regtest);
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn test_wallet_from_mnemonic() {
        Database::delete_database();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let wallet =
            Wallet::new_from_mnemonic("Test Wallet".to_string(), mnemonic, Network::Regtest)
                .unwrap();

        assert_eq!(wallet.name(), "Test Wallet");
        assert_eq!(wallet.network(), Network::Regtest);
    }

    #[test]
    fn test_address_generation() {
        let (mut wallet, _) = Wallet::new_random("Test".to_string(), Network::Regtest).unwrap();

        let addr1 = wallet.get_new_address().unwrap();
        let addr2 = wallet.get_new_address().unwrap();

        assert_ne!(addr1.as_str(), addr2.as_str());
    }

    #[test]
    fn test_wallet_balance() {
        let (wallet, _) = Wallet::new_random("Balance Test".to_string(), Network::Regtest).unwrap();
        let balance = wallet.balance();

        assert_eq!(balance.spendable(), lumo_types::Amount::ZERO);
    }

    #[test]
    fn test_metadata_persistence() {
        let test_db_path = PathBuf::from("test_metadata_persistence.db");

        // Clean up any existing test database
        let _ = std::fs::remove_file(&test_db_path);

        // Create and save wallet with fixed database path
        let mut original_wallet = Wallet::new_from_mnemonic_with_db_path(
            "Metadata Test".to_string(),
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            Network::Regtest,
            test_db_path.clone()
        ).unwrap();
        let wallet_id = original_wallet.id.clone();
        let network = original_wallet.network();

        // Generate addresses to change state
        let _addr1 = original_wallet.get_new_address().unwrap();
        let current_addr = original_wallet.get_current_address().unwrap();
        drop(original_wallet);

        // Load wallet and verify metadata persistence
        let loaded_wallet =
            Wallet::try_load_persisted_with_db_path(&wallet_id, network, test_db_path.clone())
                .unwrap();

        assert_eq!(loaded_wallet.id, wallet_id);
        assert_eq!(loaded_wallet.name(), "Metadata Test");
        assert_eq!(loaded_wallet.network(), network);
        assert_eq!(
            loaded_wallet.get_current_address().unwrap().as_str(),
            current_addr.as_str()
        );

        // Clean up test database
        let _ = std::fs::remove_file(&test_db_path);
    }

    #[test]
    fn test_load_nonexistent_wallet() {
        let fake_id = WalletId::new();
        let result = Wallet::try_load_persisted(&fake_id, Network::Regtest);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WalletError::WalletNotFound(_)
        ));
    }
}
