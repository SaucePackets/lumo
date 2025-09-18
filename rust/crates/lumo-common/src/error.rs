use thiserror::Error;

/// Common error types for Lumo wallet
#[derive(Error, Debug)]
pub enum LumoError {

    #[error("Database error: {0}")]
    Database(Box<redb::Error>),

    #[error("Serialization error: {0}")]
    Serialization(Box<serde_json::Error>),

    #[error("IO error: {0}")]
    Io(Box<std::io::Error>),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Invalid network: expected {expected}, got {actual}")]
    InvalidNetwork { expected: String, actual: String },

    #[error("Insufficient funds: need {needed}, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

/// Result type alias for Lumo operations
pub type Result<T> = std::result::Result<T, LumoError>;

impl From<redb::Error> for LumoError {
    fn from(err: redb::Error) -> Self {
        LumoError::Database(Box::new(err))
    }
}

impl From<serde_json::Error> for LumoError {
    fn from(err: serde_json::Error) -> Self {
        LumoError::Serialization(Box::new(err))
    }
}

impl From<std::io::Error> for LumoError {
    fn from(err: std::io::Error) -> Self {
        LumoError::Io(Box::new(err))
    }
}

impl From<eyre::Error> for LumoError {
    fn from(err: eyre::Error) -> Self {
        LumoError::Generic(err.to_string())
    }
}