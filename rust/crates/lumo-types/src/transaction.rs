use crate::{Address, Amount};
use bitcoin::Txid;
use derive_more::{Display, From, Into};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// Transaction ID wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
pub struct TransactionId(pub Txid);

impl TransactionId {
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, eyre::Error> {
        let txid = hex.parse::<Txid>()?;
        Ok(Self(txid))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }
}

/// Transaction direction from wallet perspective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionDirection {
    Incoming,
    Outgoing,
    SelfTransfer,
}

/// Transaction confirmation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmationStatus {
    Unconfirmed,
    Confirmed { block_height: u32 },
}

/// Basic transaction information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub amount: Amount,
    pub direction: TransactionDirection,
    pub confirmation_status: ConfirmationStatus,
    pub timestamp: Option<Timestamp>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        id: TransactionId,
        amount: Amount,
        direction: TransactionDirection,
        confirmation_status: ConfirmationStatus,
        timestamp: Option<Timestamp>,
    ) -> Self {
        Self {
            id,
            amount,
            direction,
            confirmation_status,
            timestamp,
        }
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(
            self.confirmation_status,
            ConfirmationStatus::Confirmed { .. }
        )
    }

    /// Get number of confirmations (requires current block height)
    pub fn confirmations(&self, current_height: u32) -> u32 {
        match self.confirmation_status {
            ConfirmationStatus::Unconfirmed => 0,
            ConfirmationStatus::Confirmed { block_height } => {
                current_height.saturating_sub(block_height) + 1
            }
        }
    }
}

/// Detailed transaction information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub transaction: Transaction,
    pub fee: Option<Amount>,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub note: Option<String>,
}

/// Transaction input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: TransactionId,
    pub previous_output_index: u32,
    pub amount: Amount,
    pub address: Option<Address>,
    pub is_mine: bool,
}

/// Transaction output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub index: u32,
    pub amount: Amount,
    pub address: Option<Address>,
    pub is_mine: bool,
    pub is_change: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let txid = TransactionId::from_hex(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let amount = Amount::from_sat(100_000);
        let tx = Transaction::new(
            txid,
            amount,
            TransactionDirection::Incoming,
            ConfirmationStatus::Unconfirmed,
            None,
        );

        assert_eq!(tx.id, txid);
        assert_eq!(tx.amount, amount);
        assert!(!tx.is_confirmed());
    }

    #[test]
    fn test_confirmations() {
        let txid = TransactionId::from_hex(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let tx = Transaction::new(
            txid,
            Amount::from_sat(100_000),
            TransactionDirection::Incoming,
            ConfirmationStatus::Confirmed { block_height: 100 },
            None,
        );

        assert_eq!(tx.confirmations(102), 3);
        assert!(tx.is_confirmed());
    }
}
