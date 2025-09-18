use bitcoin::Network as BitcoinNetwork;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

/// Bitcoin network types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, From, Serialize, Deserialize, Default)]
pub enum Network {
    /// Bitcoin mainnet
    #[display("mainnet")]
    #[default]
    Bitcoin,
    /// Bitcoin testnet
    #[display("testnet")]
    Testnet,
    /// Bitcoin signet
    #[display("signet")]
    Signet,
    /// Bitcoin regtest
    #[display("regtest")]
    Regtest,
}

impl Network {
    /// Convert to bitcoin crate's Network type
    pub fn to_bitcoin_network(&self) -> BitcoinNetwork {
        match self {
            Network::Bitcoin => BitcoinNetwork::Bitcoin,
            Network::Testnet => BitcoinNetwork::Testnet,
            Network::Signet => BitcoinNetwork::Signet,
            Network::Regtest => BitcoinNetwork::Regtest,
        }
    }

    /// Create from bitcoin crate's Network type
    pub fn from_bitcoin_network(network: BitcoinNetwork) -> Self {
        match network {
            BitcoinNetwork::Bitcoin => Network::Bitcoin,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Signet => Network::Signet,
            BitcoinNetwork::Regtest => Network::Regtest,
            BitcoinNetwork::Testnet4 => Network::Testnet, // Map testnet4 to testnet
        }
    }

    /// Check if this is a test network
    pub fn is_testnet(&self) -> bool {
        matches!(self, Network::Testnet | Network::Signet | Network::Regtest)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_conversion() {
        let network = Network::Bitcoin;
        let bitcoin_network = network.to_bitcoin_network();
        let back = Network::from_bitcoin_network(bitcoin_network);
        assert_eq!(network, back);
    }

    #[test]
    fn test_is_testnet() {
        assert!(!Network::Bitcoin.is_testnet());
        assert!(Network::Testnet.is_testnet());
        assert!(Network::Signet.is_testnet());
        assert!(Network::Regtest.is_testnet());
    }
}