pub mod bdk_store;
pub mod node;
pub mod node_urls;
pub mod wallet;

// Re-export types from our crates
pub use lumo_common::{setup_logging, LumoError, GAP_LIMIT, MIN_SEND_SATS, ROOT_DATA_DIR};
pub use lumo_types::*;

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
    fn test_wallet_demo() {
        // Initialize
        init().unwrap();

        println!("\n🚀 Lumo Wallet Demo");
        println!("==================");

        // Create a new random wallet
        println!("\n1. Creating a new random wallet...");
        let (mut wallet, mnemonic) =
            Wallet::new_random("My Test Wallet".to_string(), Network::Regtest).unwrap();

        println!("✅ Wallet created successfully!");
        println!("   ID: {}", wallet.id);
        println!("   Name: {}", wallet.name());
        println!("   Network: {:?}", wallet.network());

        // Show the mnemonic
        println!("\n2. Generated mnemonic phrase:");
        println!("   {mnemonic}");

        // Generate some receiving addresses
        println!("\n3. Generating receiving addresses:");
        for i in 1..=3 {
            let address = wallet.get_new_address().unwrap();
            println!("   Address {}: {}", i, address.as_str());
        }

        // Show current address
        println!("\n4. Current receiving address:");
        let current_addr = wallet.get_current_address().unwrap();
        println!("   {}", current_addr.as_str());

        println!("\n🎉 Wallet demo completed!");
    }
}
