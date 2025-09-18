use std::path::PathBuf;
use bitcoin::Amount;
use once_cell::sync::Lazy;

/// Static data directories - computed once at startup
pub static ROOT_DATA_DIR: Lazy<PathBuf> = Lazy::new(data_dir_init);
pub static WALLET_DATA_DIR: Lazy<PathBuf> = Lazy::new(wallet_data_dir_init);

/// Bitcoin wallet constants
pub static GAP_LIMIT: u8 = 30;
pub static MIN_SEND_SATS: u64 = 5000;
pub static MIN_SEND_AMOUNT: Amount = Amount::from_sat(MIN_SEND_SATS);

/// Dust limit for Bitcoin transactions
pub static DUST_LIMIT_SATS: u64 = 546;
pub static DUST_LIMIT_AMOUNT: Amount = Amount::from_sat(DUST_LIMIT_SATS);

fn data_dir_init() -> PathBuf {
    let dir = dirs::data_dir()
        .expect("failed to get data directory")
        .join("lumo");

    init_dir(dir).expect("failed to initialize data directory")
}

fn wallet_data_dir_init() -> PathBuf {
    let dir = ROOT_DATA_DIR.join("wallets");
    init_dir(dir).expect("failed to initialize wallet data directory")
}

fn init_dir(dir: PathBuf) -> Result<PathBuf, std::io::Error> {
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}