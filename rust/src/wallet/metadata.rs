use std::str::FromStr;

use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lumo_types::Network;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
pub struct WalletId(Uuid);

impl WalletId {
    pub fn new() -> Self {
        WalletId(Uuid::new_v4())
    }

    /// Create from string
    pub fn from_string(s: &str) -> crate::wallet::error::Result<Self> {
        let uuid = Uuid::from_str(s).map_err(|_| {
            crate::wallet::error::WalletError::Generic(format!("Invalid wallet ID: {s}"))
        })?;
        Ok(Self(uuid))
    }
}

impl Default for WalletId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletType {
    #[default]
    Hot,
    Cold,
    XpubOnly,
}

impl WalletType {
    // Check if the wallet type can sign transactions
    pub fn can_sign(&self) -> bool {
        match self {
            WalletType::Hot => true,
            WalletType::Cold => true,
            WalletType::XpubOnly => false,
        }
    }

    // Wallet type description
    pub fn description(&self) -> &'static str {
        match self {
            WalletType::Hot => "Hot wallet, software wallet (can sign transactions)",
            WalletType::Cold => "Hardware wallet (requires device to sign)",
            WalletType::XpubOnly => "Watch-only wallet (cannot sign transactions)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    pub id: WalletId,
    pub name: String,
    pub network: Network,
    pub created_at: String, // ISO timestamp
    #[serde(default)]
    pub wallet_type: WalletType,
    pub master_fingerprint: Option<String>,
}

impl WalletMetadata {
    pub fn new(name: String, network: Network) -> Self {
        Self {
            id: WalletId::new(),
            name,
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
            wallet_type: WalletType::Hot, // Default to Hot wallet
            master_fingerprint: None,
        }
    }

    pub fn new_for_hardware(
        id: WalletId,
        name: String,
        network: Network,
        fingerprint: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
            wallet_type: WalletType::Cold,
            master_fingerprint: fingerprint,
        }
    }

    pub fn new_imported_from_mnemonic(
        id: WalletId,
        name: String,
        network: Network,
        fingerprint: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
            wallet_type: WalletType::Hot,
            master_fingerprint: fingerprint,
        }
    }

    pub fn new_from_xpub(
        id: WalletId,
        name: String,
        network: Network,
        fingerprint: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            network,
            created_at: chrono::Utc::now().to_rfc3339(),
            wallet_type: match &fingerprint {
                Some(_) => WalletType::Cold,
                None => WalletType::XpubOnly,
            },
            master_fingerprint: fingerprint,
        }
    }
}
