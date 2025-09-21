use bitcoin::Network as BitcoinNetwork;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

/// Bitcoin network types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, From, Serialize, Deserialize, Default,
)]
pub enum Network {
    /// Bitcoin mainnet
    #[display("mainnet")]
    #[default]
    Bitcoin,
    /// Bitcoin testnet (legacy testnet3)
    #[display("testnet")]
    Testnet,
    /// Bitcoin testnet4 (new testnet)
    #[display("testnet4")]
    Testnet4,
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
            Network::Testnet4 => BitcoinNetwork::Testnet4,
            Network::Signet => BitcoinNetwork::Signet,
            Network::Regtest => BitcoinNetwork::Regtest,
        }
    }

    /// Create from bitcoin crate's Network type
    pub fn from_bitcoin_network(network: BitcoinNetwork) -> Self {
        match network {
            BitcoinNetwork::Bitcoin => Network::Bitcoin,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Testnet4 => Network::Testnet4,
            BitcoinNetwork::Signet => Network::Signet,
            BitcoinNetwork::Regtest => Network::Regtest,
        }
    }

    /// Check if this is a test network
    pub fn is_testnet(&self) -> bool {
        matches!(
            self,
            Network::Testnet | Network::Testnet4 | Network::Signet | Network::Regtest
        )
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
        assert!(Network::Testnet4.is_testnet());
        assert!(Network::Signet.is_testnet());
        assert!(Network::Regtest.is_testnet());
    }

    #[test]
    fn test_testnet4_conversion() {
        let testnet4 = Network::Testnet4;
        let bitcoin_network = testnet4.to_bitcoin_network();
        assert_eq!(bitcoin_network, BitcoinNetwork::Testnet4);

        let back = Network::from_bitcoin_network(bitcoin_network);
        assert_eq!(back, Network::Testnet4);
    }
}
