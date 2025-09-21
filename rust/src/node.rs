pub mod client;
use crate::node_urls::*;
use lumo_types::Network;

pub struct Node {
    pub name: String,
    pub network: Network,
    pub url: String,
}

impl Node {
    pub fn default(network: Network) -> Self {
        match network {
            Network::Bitcoin => {
                let (name, url) = BITCOIN_ESPLORA[0];

                Self {
                    name: name.to_string(),
                    network,
                    url: url.to_string(),
                }
            }
            Network::Testnet => {
                let (name, url) = TESTNET_ESPLORA[0];
                Self {
                    name: name.to_string(),
                    network,
                    url: url.to_string(),
                }
            }
            Network::Testnet4 => {
                let (name, url) = TESTNET4_ESPLORA[0];
                Self {
                    name: name.to_string(),
                    network,
                    url: url.to_string(),
                }
            }
            Network::Regtest => {
                let (name, url) = REGTEST_ESPLORA[0];
                Self {
                    name: name.to_string(),
                    network,
                    url: url.to_string(),
                }
            }
            Network::Signet => {
                let (name, url) = SIGNET_ESPLORA[0];
                Self {
                    name: name.to_string(),
                    network,
                    url: url.to_string(),
                }
            }
        }
    }
}
