use crate::database::error::DatabaseError;
use bdk_wallet::descriptor::DescriptorError;
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

    #[error("Wallet already exists with ID: {0}")]
    WalletAlreadyExists(String),
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

impl From<bitcoin::bip32::Error> for WalletError {
    fn from(err: bitcoin::bip32::Error) -> Self {
        WalletError::Bitcoin(err.to_string())
    }
}

impl From<DescriptorError> for WalletError {
    fn from(err: DescriptorError) -> Self {
        WalletError::Bdk(err.to_string())
    }
}

impl From<bdk_wallet::CreateWithPersistError<bdk_wallet::rusqlite::Error>> for WalletError {
    fn from(err: bdk_wallet::CreateWithPersistError<bdk_wallet::rusqlite::Error>) -> Self {
        WalletError::Bdk(err.to_string())
    }
}

impl From<bdk_wallet::LoadWithPersistError<bdk_wallet::rusqlite::Error>> for WalletError {
    fn from(err: bdk_wallet::LoadWithPersistError<bdk_wallet::rusqlite::Error>) -> Self {
        WalletError::Bdk(err.to_string())
    }
}

/// Result type alias for wallet operations
pub type Result<T> = std::result::Result<T, WalletError>;
