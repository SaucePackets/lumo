use redb::{CommitError, StorageError, TableError, TransactionError};

#[derive(Debug, Clone, Hash, PartialEq, Eq, thiserror::Error)]
pub enum DatabaseError {
    #[error("Failed to open database: {0}")]
    OpenError(String),

    #[error("Failed to read database: {0}")]
    ReadError(String),

    #[error("Failed to write database: {0}")]
    WriteError(String),

    #[error("ReDB error: {0}")]
    RedbError(String),

    #[error("JSON Serialization error: {0}")]
    SerializationError(String),
}

impl From<redb::DatabaseError> for DatabaseError {
    fn from(err: redb::DatabaseError) -> Self {
        DatabaseError::RedbError(err.to_string())
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::SerializationError(err.to_string())
    }
}

impl From<TransactionError> for DatabaseError {
    fn from(err: TransactionError) -> Self {
        DatabaseError::RedbError(err.to_string())
    }
}

impl From<TableError> for DatabaseError {
    fn from(err: TableError) -> Self {
        DatabaseError::RedbError(err.to_string())
    }
}

impl From<StorageError> for DatabaseError {
    fn from(err: StorageError) -> Self {
        DatabaseError::RedbError(err.to_string())
    }
}

impl From<CommitError> for DatabaseError {
    fn from(err: CommitError) -> Self {
        DatabaseError::WriteError(err.to_string())
    }
}

impl From<crate::wallet::error::WalletError> for DatabaseError {
    fn from(err: crate::wallet::error::WalletError) -> Self {
        DatabaseError::ReadError(err.to_string())
    }
}
