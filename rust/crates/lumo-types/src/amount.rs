use bitcoin::Amount as BdkAmount;
use derive_more::{Add, Deref, Display, From, Into, Sub};
use serde::{Deserialize, Serialize};

/// Bitcoin amount wrapper using BDK's Amount type
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Add,
    Sub,
    Display,
    From,
    Into,
    Deref,
    Serialize,
    Deserialize,
)]
pub struct Amount(pub BdkAmount);

impl Amount {
    /// Constants from BDK
    pub const ZERO: Amount = Amount(BdkAmount::ZERO);
    pub const ONE_SAT: Amount = Amount(BdkAmount::ONE_SAT);
    pub const ONE_BTC: Amount = Amount(BdkAmount::ONE_BTC);
    pub const MAX_MONEY: Amount = Amount(BdkAmount::MAX_MONEY);

    /// Create a new Amount from satoshis
    pub const fn from_sat(satoshis: u64) -> Self {
        Self(BdkAmount::from_sat(satoshis))
    }

    /// Create a new Amount from Bitcoin (uses BDK's validation)
    pub fn from_btc(btc: f64) -> Result<Self, eyre::Error> {
        let bdk_amount = BdkAmount::from_btc(btc)?;
        Ok(Self(bdk_amount))
    }

    /// Get the amount in satoshis
    pub fn as_sat(&self) -> u64 {
        self.0.to_sat()
    }

    /// Get the amount in Bitcoin
    pub fn as_btc(&self) -> f64 {
        self.0.to_btc()
    }

    /// Check if the amount is zero
    pub fn is_zero(&self) -> bool {
        self.0 == BdkAmount::ZERO
    }

    /// Convert to BDK's Amount type
    pub fn to_bdk_amount(&self) -> BdkAmount {
        self.0
    }

    /// Create from BDK's Amount type
    pub fn from_bdk_amount(amount: BdkAmount) -> Self {
        Self(amount)
    }

    /// Check if amount is dust (below minimum spendable amount)
    pub fn is_dust(&self) -> bool {
        self.0 < BdkAmount::from_sat(546) // Standard dust limit
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_creation() {
        let amount = Amount::from_sat(100_000_000);
        assert_eq!(amount.as_sat(), 100_000_000);
        assert_eq!(amount.as_btc(), 1.0);
    }

    #[test]
    fn test_amount_from_btc() {
        let amount = Amount::from_btc(1.5).unwrap();
        assert_eq!(amount.as_sat(), 150_000_000);
    }

    #[test]
    fn test_amount_addition() {
        let a = Amount::from_sat(100);
        let b = Amount::from_sat(200);
        let result = a + b;
        assert_eq!(result.as_sat(), 300);
    }

    #[test]
    fn test_amount_constants() {
        assert_eq!(Amount::ZERO.as_sat(), 0);
        assert_eq!(Amount::ONE_SAT.as_sat(), 1);
        assert_eq!(Amount::ONE_BTC.as_sat(), 100_000_000);
    }

    #[test]
    fn test_dust_detection() {
        assert!(Amount::from_sat(100).is_dust());
        assert!(!Amount::from_sat(1000).is_dust());
    }

    #[test]
    fn test_bdk_conversion() {
        let bdk_amount = BdkAmount::from_sat(50000);
        let amount = Amount::from_bdk_amount(bdk_amount);
        assert_eq!(amount.to_bdk_amount(), bdk_amount);
    }
}
