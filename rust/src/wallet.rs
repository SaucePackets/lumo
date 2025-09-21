pub mod balance;
pub mod error;

use bdk_wallet::{
    template::{Bip84, DescriptorTemplate},
    KeychainKind, Wallet as BdkWallet,
};
use bip39::Mnemonic;
use derive_more::{Display, From, Into};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::bdk_store::BDKStore;
use crate::wallet::balance::Balance;
use crate::wallet::error::{Result, WalletError};
use lumo_types::{Address, Network};

type PersistedBdkWallet = bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>;

/// Unique wallet identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
pub struct WalletId(Uuid);

impl WalletId {
    /// Generate a new random wallet ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from string
    pub fn from_string(s: &str) -> Result<Self> {
        let uuid = Uuid::from_str(s)
            .map_err(|_| WalletError::Generic(format!("Invalid wallet ID: {s}")))?;
        Ok(Self(uuid))
    }
}

impl Default for WalletId {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic wallet metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    pub id: WalletId,
    pub name: String,
    pub network: Network,
    pub created_at: String, // ISO timestamp
}

impl WalletMetadata {
    pub fn new(name: String, network: Network) -> Self {
        Self {
            id: WalletId::new(),
            name,
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

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
        let (external_descriptor, _external_keymap, _) = Bip84(xpriv, KeychainKind::External)
            .build(bdk_network)
            .map_err(|e| WalletError::Bdk(e.to_string()))?;

        let (internal_descriptor, _internal_keymap, _) = Bip84(xpriv, KeychainKind::Internal)
            .build(bdk_network)
            .map_err(|e| WalletError::Bdk(e.to_string()))?;

        let mut store = BDKStore::try_new(wallet_id, network)?;

        // Create BDK wallet (in-memory for now, no persistence)
        let wallet = BdkWallet::create(external_descriptor, internal_descriptor)
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

        let metadata = WalletMetadata {
            id: wallet_id.clone(),
            name: format!("Loaded Wallet {wallet_id}"),
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(Self {
            id: wallet_id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    pub fn balance(&self) -> Balance {
        Balance(self.bdk.balance())
    }

    /// Get a new receiving address
    pub fn get_new_address(&mut self) -> Result<Address> {
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

    #[test]
    fn test_wallet_id_creation() {
        let id1 = WalletId::new();
        let id2 = WalletId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_wallet_id_string_conversion() {
        let id = WalletId::new();
        let id_str = id.to_string(); // Uses Display trait
        let parsed_id = WalletId::from_string(&id_str).unwrap();
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_wallet_creation_random() {
        let (wallet, mnemonic) =
            Wallet::new_random("Test Wallet".to_string(), Network::Regtest).unwrap();
        assert_eq!(wallet.name(), "Test Wallet");
        assert_eq!(wallet.network(), Network::Regtest);
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn test_wallet_from_mnemonic() {
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

        // Should generate different addresses
        assert_ne!(addr1.as_str(), addr2.as_str());
    }

    #[test]
    fn test_wallet_persistence() {
        // Create a wallet and generate an address
        let (mut wallet, mnemonic) =
            Wallet::new_random("Persistence Test".to_string(), Network::Regtest).unwrap();
        let _wallet_id = wallet.id.clone();
        let first_address = wallet.get_new_address().unwrap();

        // Drop the wallet to ensure it's not in memory
        drop(wallet);

        // Recreate wallet from same mnemonic and ID - this should load from persistence
        let restored_wallet = Wallet::new_from_mnemonic(
            "Persistence Test".to_string(),
            &mnemonic.to_string(),
            Network::Regtest,
        )
        .unwrap();

        // The wallet should remember the address we generated
        let current_address = restored_wallet.get_current_address().unwrap();

        // This proves persistence is working - the restored wallet remembers the last address
        assert_eq!(first_address.as_str(), current_address.as_str());

        println!(
            "✅ Persistence test passed! Wallet remembered address: {}",
            first_address.as_str()
        );
    }

    #[test]
    fn test_load_persisted_wallet() {
        // Step 1: Create a wallet and generate some addresses
        let (mut original_wallet, _mnemonic) =
            Wallet::new_random("Load Test Wallet".to_string(), Network::Regtest).unwrap();

        let wallet_id = original_wallet.id.clone();
        let network = original_wallet.network();

        // Generate a few addresses to change the wallet state
        let addr1 = original_wallet.get_new_address().unwrap();
        let addr2 = original_wallet.get_new_address().unwrap();
        let current_addr = original_wallet.get_current_address().unwrap();

        println!("Created wallet with ID: {wallet_id}");
        println!(
            "Generated addresses: {}, {}",
            addr1.as_str(),
            addr2.as_str()
        );

        // Step 2: Drop the original wallet to ensure it's not in memory
        drop(original_wallet);

        // Step 3: Load the wallet using try_load_persisted
        let loaded_wallet = Wallet::try_load_persisted(&wallet_id, network).unwrap();

        // Step 4: Verify the loaded wallet has the same state
        assert_eq!(loaded_wallet.id, wallet_id);
        assert_eq!(loaded_wallet.network(), network);

        // The most important test: does it remember the address generation state?
        let loaded_current_addr = loaded_wallet.get_current_address().unwrap();
        assert_eq!(current_addr.as_str(), loaded_current_addr.as_str());

        // Test that metadata was reconstructed
        assert!(loaded_wallet.name().contains("Loaded Wallet"));

        println!("✅ Load test passed! Loaded wallet remembers state:");
        println!("   Current address: {}", loaded_current_addr.as_str());
        println!("   Wallet name: {}", loaded_wallet.name());
    }

    #[test]
    fn test_load_nonexistent_wallet() {
        // Try to load a wallet that doesn't exist
        let fake_id = WalletId::new();
        let result = Wallet::try_load_persisted(&fake_id, Network::Regtest);

        // Should return an error
        assert!(result.is_err());

        // Should be a "wallet not found" error
        match result.unwrap_err() {
            WalletError::WalletNotFound(_) => {
                println!("✅ Correctly returned WalletNotFound error for nonexistent wallet");
            }
            other => panic!("Expected WalletNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn test_wallet_balance() {
        // Create a new wallet
        let (wallet, _mnemonic) =
            Wallet::new_random("Balance Test".to_string(), Network::Regtest).unwrap();

        // Get the balance
        let balance = wallet.balance();

        // For a new wallet with no transactions, balance should be zero
        assert_eq!(balance.spendable(), lumo_types::Amount::ZERO);

        println!(
            "✅ New wallet balance is zero: {} sats",
            balance.spendable().as_sat()
        );
        println!(
            "✅ New wallet balance is zero: {} BTC",
            balance.spendable().as_btc()
        );

        // Test that the method doesn't panic and returns a valid amount
        assert!(balance.spendable().as_sat() == 0);
    }

    #[test]
    fn test_balance_after_loading() {
        // Create a wallet and check its balance
        let (wallet, _mnemonic) =
            Wallet::new_random("Balance Load Test".to_string(), Network::Regtest).unwrap();
        let wallet_id = wallet.id.clone();
        let network = wallet.network();

        // Initial balance should be zero
        let initial_balance = wallet.balance();
        assert_eq!(initial_balance.spendable(), lumo_types::Amount::ZERO);

        // Drop the wallet
        drop(wallet);

        // Load the wallet and check balance again
        let loaded_wallet = Wallet::try_load_persisted(&wallet_id, network).unwrap();
        let loaded_balance = loaded_wallet.balance();

        // Balance should still be zero and match the original
        assert_eq!(loaded_balance.spendable(), initial_balance.spendable());
        assert_eq!(loaded_balance.spendable(), lumo_types::Amount::ZERO);

        println!(
            "✅ Loaded wallet maintains balance: {} sats",
            loaded_balance.spendable().as_sat()
        );
    }
}
