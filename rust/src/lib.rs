pub mod bdk_store;
pub mod database;
pub mod node;
pub mod node_urls;
pub mod wallet;
pub mod wallet_manager;
pub mod fee_estimation;

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
}
