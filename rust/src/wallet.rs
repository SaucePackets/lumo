pub mod error;

use bdk_wallet::{KeychainKind, Wallet as BdkWallet, template::{Bip84, DescriptorTemplate}};
use bip39::Mnemonic;
use rand::Rng;
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use lumo_types::{Network, Address};
use crate::wallet::error::{WalletError, Result};

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
pub struct Wallet {
    pub id: WalletId,
    pub metadata: WalletMetadata,
    pub bdk: BdkWallet,
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
        let bdk_wallet = Self::create_bdk_wallet(&mnemonic, network, None)?;

        Ok(Self {
            id: metadata.id.clone(),
            metadata,
            bdk: bdk_wallet,
        })
    }

    /// Create a new wallet with random mnemonic
    pub fn new_random(name: String, network: Network) -> Result<(Self, Mnemonic)> {
        // Generate random mnemonic (24 words = 256 bits = 32 bytes)
        let random_bytes = rand::rng().random::<[u8; 32]>();
        let mnemonic = Mnemonic::from_entropy(&random_bytes)
            .map_err(WalletError::InvalidMnemonic)?;

        // Create metadata
        let metadata = WalletMetadata::new(name, network);

        // Create BDK wallet
        let bdk_wallet = Self::create_bdk_wallet(&mnemonic, network, None)?;

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
        passphrase: Option<&str>,
    ) -> Result<BdkWallet> {
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

        // Create BDK wallet (in-memory for now, no persistence)
        let wallet = BdkWallet::create(external_descriptor, internal_descriptor)
            .network(bdk_network)
            .create_wallet_no_persist()
            .map_err(|e| WalletError::Bdk(e.to_string()))?;

        Ok(wallet)
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
        let (wallet, mnemonic) = Wallet::new_random("Test Wallet".to_string(), Network::Regtest).unwrap();
        assert_eq!(wallet.name(), "Test Wallet");
        assert_eq!(wallet.network(), Network::Regtest);
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn test_wallet_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let wallet = Wallet::new_from_mnemonic(
            "Test Wallet".to_string(),
            mnemonic,
            Network::Regtest,
        ).unwrap();

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
}