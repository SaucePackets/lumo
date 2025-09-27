pub mod bdk_store;
pub mod database;
pub mod node;
pub mod node_urls;
pub mod wallet;
pub mod wallet_manager;

// Re-export types from our crates
pub use lumo_common::{setup_logging, LumoError, GAP_LIMIT, MIN_SEND_SATS, ROOT_DATA_DIR};
pub use lumo_types::*;
pub use wallet_manager::WalletManager;

// Re-export wallet types
pub use wallet::{
    error::{Result as WalletResult, WalletError},
    Wallet, WalletId, WalletMetadata,
};

/// Initialize the Lumo wallet library
pub fn init() -> lumo_common::Result<()> {
    lumo_common::setup_logging()?;
    tracing::info!("Lumo wallet library initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_basic_wallet_functionality() {
        database::Database::delete_database();
        init().unwrap();

        let (mut wallet, mnemonic) =
            Wallet::new_random("Test Wallet".to_string(), Network::Regtest).unwrap();

        assert_eq!(wallet.name(), "Test Wallet");
        assert_eq!(wallet.network(), Network::Regtest);
        assert_eq!(mnemonic.word_count(), 12);

        let addr1 = wallet.get_new_address().unwrap();
        let addr2 = wallet.get_new_address().unwrap();
        let current_addr = wallet.get_current_address().unwrap();

        assert_ne!(addr1.as_str(), addr2.as_str());
        assert!(!current_addr.as_str().is_empty());
    }
}
