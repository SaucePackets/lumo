use crate::{Amount, Network};
use bdk_wallet::chain::bitcoin::Address as BdkAddress;
use bitcoin::address::{NetworkChecked, NetworkUnchecked};
use bitcoin::params::Params;
use derive_more::{Deref, Display, From, Into};
use serde::{Deserialize, Serialize};

/// Bitcoin address wrapper using BDK's address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Deref, Serialize)]
pub struct Address(BdkAddress<NetworkChecked>);

/// Address with network information and optional amount (for BIP21 URIs)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressWithNetwork {
    pub address: Address,
    pub network: Network,
    pub amount: Option<Amount>,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: String,
    pub index: u32,
    pub is_used: bool,
    pub balance: Amount,
}

/// Address validation errors
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AddressError {
    #[error("Invalid address format")]
    InvalidFormat,

    #[error("Address is valid but for wrong network - expected {expected}, got {actual}")]
    WrongNetwork { expected: Network, actual: Network },

    #[error("Address is valid but for unsupported network")]
    UnsupportedNetwork,

    #[error("Empty address string")]
    EmptyAddress,

    #[error("Invalid amount in BIP21 URI: {0}")]
    InvalidAmount(String),
}

impl Address {
    /// Create address from BDK address
    pub fn new(address: BdkAddress<NetworkChecked>) -> Self {
        Self(address)
    }

    /// Create address from string with network validation
    pub fn from_string(address_str: &str, network: Network) -> Result<Self, AddressError> {
        let address_str = address_str.trim();
        if address_str.is_empty() {
            return Err(AddressError::EmptyAddress);
        }

        // Parse as unchecked first
        let unchecked: BdkAddress<NetworkUnchecked> = address_str
            .parse()
            .map_err(|_| AddressError::InvalidFormat)?;

        // Check if valid for the requested network
        let bitcoin_network = network.to_bitcoin_network();
        if unchecked.is_valid_for_network(bitcoin_network) {
            let checked = unchecked
                .require_network(bitcoin_network)
                .expect("just validated");
            return Ok(Self::new(checked));
        }

        // Check what network it's actually valid for
        for test_network in [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ] {
            if unchecked.is_valid_for_network(test_network.to_bitcoin_network()) {
                return Err(AddressError::WrongNetwork {
                    expected: network,
                    actual: test_network,
                });
            }
        }

        Err(AddressError::UnsupportedNetwork)
    }

    /// Get the address as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Convert to BDK address
    pub fn to_bdk_address(&self) -> &BdkAddress<NetworkChecked> {
        &self.0
    }

    /// Convert to unchecked address
    pub fn into_unchecked(self) -> BdkAddress<NetworkUnchecked> {
        self.0.into_unchecked()
    }

    /// Create from script and network params
    pub fn from_script(script: &bitcoin::Script, params: Params) -> Result<Self, AddressError> {
        let address =
            BdkAddress::from_script(script, params).map_err(|_| AddressError::InvalidFormat)?;
        Ok(Self::new(address))
    }
}

impl AddressWithNetwork {
    /// Parse address with automatic network detection and BIP21 support
    pub fn from_string(input: &str) -> Result<Self, AddressError> {
        let input = input.trim();

        // Handle bitcoin: URI prefix
        let input = input.strip_prefix("bitcoin:").unwrap_or(input);

        // Extract address and amount from BIP21 URI
        let (address_str, amount) = extract_amount_from_uri(input)?;

        // Parse as unchecked to detect network
        let unchecked: BdkAddress<NetworkUnchecked> = address_str
            .parse()
            .map_err(|_| AddressError::InvalidFormat)?;

        // Try each network to find the correct one
        for network in [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ] {
            if unchecked.is_valid_for_network(network.to_bitcoin_network()) {
                let checked = unchecked
                    .clone()
                    .require_network(network.to_bitcoin_network())
                    .expect("just validated");

                return Ok(Self {
                    address: Address::new(checked),
                    network,
                    amount,
                });
            }
        }

        Err(AddressError::UnsupportedNetwork)
    }

    /// Check if address is valid for given network
    pub fn is_valid_for_network(&self, network: Network) -> bool {
        // Use network kind comparison for mainnet/testnet compatibility
        let current_kind = bitcoin::NetworkKind::from(self.network.to_bitcoin_network());
        let target_kind = bitcoin::NetworkKind::from(network.to_bitcoin_network());
        current_kind == target_kind
    }
}

/// Extract amount from BIP21 URI (bitcoin:address?amount=0.001)
fn extract_amount_from_uri(uri: &str) -> Result<(&str, Option<Amount>), AddressError> {
    // Find the ?amount= part
    let Some(amount_pos) = uri.find("?amount=") else {
        return Ok((uri, None));
    };

    let address_part = &uri[..amount_pos];
    let amount_start = amount_pos + 8; // Skip "?amount="

    // Find the end of the amount (next & or end of string)
    let amount_end = uri[amount_start..]
        .find('&')
        .map(|pos| amount_start + pos)
        .unwrap_or(uri.len());

    let amount_str = &uri[amount_start..amount_end];

    // Parse the amount
    let amount_btc: f64 = amount_str
        .parse()
        .map_err(|_| AddressError::InvalidAmount(amount_str.to_string()))?;

    let amount = Amount::from_btc(amount_btc)
        .map_err(|_| AddressError::InvalidAmount(amount_str.to_string()))?;

    Ok((address_part, Some(amount)))
}

/// Validate address string for given network
pub fn validate_address(address_str: &str, network: Network) -> Result<(), AddressError> {
    Address::from_string(address_str, network)?;
    Ok(())
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let unchecked: BdkAddress<NetworkUnchecked> =
            s.parse().map_err(serde::de::Error::custom)?;

        // For deserialization, assume checked (used for stored addresses)
        let checked = unchecked.assume_checked();
        Ok(Address::new(checked))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_from_string() {
        let addr_str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let address = Address::from_string(addr_str, Network::Bitcoin).unwrap();
        assert_eq!(address.as_str(), addr_str);
    }

    #[test]
    fn test_address_wrong_network() {
        let addr_str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"; // mainnet address
        let result = Address::from_string(addr_str, Network::Testnet);
        assert!(matches!(result, Err(AddressError::WrongNetwork { .. })));
    }

    #[test]
    fn test_address_with_network_detection() {
        let addr_str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let address_with_network = AddressWithNetwork::from_string(addr_str).unwrap();
        assert_eq!(address_with_network.network, Network::Bitcoin);
        assert_eq!(address_with_network.amount, None);
    }

    #[test]
    fn test_bip21_uri_parsing() {
        let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.001";
        let address_with_network = AddressWithNetwork::from_string(uri).unwrap();
        assert_eq!(address_with_network.network, Network::Bitcoin);
        assert_eq!(
            address_with_network.amount,
            Some(Amount::from_btc(0.001).unwrap())
        );
    }

    #[test]
    fn test_bip21_uri_with_other_params() {
        let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.002&label=test";
        let address_with_network = AddressWithNetwork::from_string(uri).unwrap();
        assert_eq!(
            address_with_network.amount,
            Some(Amount::from_btc(0.002).unwrap())
        );
    }

    #[test]
    fn test_validate_address() {
        let addr_str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        assert!(validate_address(addr_str, Network::Bitcoin).is_ok());
        assert!(validate_address(addr_str, Network::Testnet).is_err());
    }

    #[test]
    fn test_empty_address() {
        assert!(matches!(
            Address::from_string("", Network::Bitcoin),
            Err(AddressError::EmptyAddress)
        ));
    }
}
