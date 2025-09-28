pub const MAINNET_ESPLORA: [(&str, &str); 2] = [
    ("blockstream.info", "https://blockstream.info/api/"),
    ("mempool.space", "https://mempool.space/api/"),
];

pub const MAINNET_ELECTRUM: [(&str, &str); 3] = [
    (
        "electrum.blockstream.info",
        "ssl://electrum.blockstream.info:50002",
    ),
    ("mempool.space electrum", "ssl://mempool.space:50002"),
    ("electrum.diynodes.com", "ssl://electrum.diynodes.com:50022"),
];

pub const TESTNET_ESPLORA: [(&str, &str); 2] = [
    ("mempool.space", "https://mempool.space/testnet/api/"),
    ("blockstream.info", "https://blockstream.info/testnet/api/"),
];

pub const TESTNET_ELECTRUM: [(&str, &str); 1] =
    [("testnet.hsmiths.com", "ssl://testnet.hsmiths.com:53012")];

pub const TESTNET4_ESPLORA: [(&str, &str); 1] =
    [("mempool.space", "https://mempool.space/testnet4/api/")];

pub const TESTNET4_ELECTRUM: [(&str, &str); 1] =
    [("mempool.space electrum", "ssl://mempool.space:40002")];

pub const REGTEST_ESPLORA: [(&str, &str); 1] = [
    ("local", "http://localhost:3002"), // For local development
];

pub const SIGNET_ESPLORA: [(&str, &str); 1] =
    [("mempool.space", "https://mempool.space/signet/api/")];

use lumo_types::Network;

pub fn default_esplora_urls(network: Network) -> &'static str {
    match network {
        Network::Mainnet => MAINNET_ESPLORA[0].1,
        Network::Testnet => TESTNET_ESPLORA[0].1,
        Network::Testnet4 => TESTNET4_ESPLORA[0].1,
        Network::Regtest => REGTEST_ESPLORA[0].1,
        Network::Signet => SIGNET_ESPLORA[0].1,
    }
}
