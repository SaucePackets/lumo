use crate::database::error::DatabaseError;
use thiserror::Error;

/// Wallet operation errors
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("BDK wallet error: {0}")]
    Bdk(String),

    #[error("Bitcoin error: {0}")]
    Bitcoin(String),

    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(#[from] bip39::Error),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Invalid network: {0}")]
    InvalidNetwork(String),

    #[error("Address generation failed: {0}")]
    AddressGeneration(String),

    #[error("Generic wallet error: {0}")]
    Generic(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<eyre::Error> for WalletError {
    fn from(err: eyre::Error) -> Self {
        WalletError::Generic(err.to_string())
    }
}

impl From<DatabaseError> for WalletError {
    fn from(err: DatabaseError) -> Self {
        WalletError::Database(err.to_string())
    }
}

/// Result type alias for wallet operations
pub type Result<T> = std::result::Result<T, WalletError>;
