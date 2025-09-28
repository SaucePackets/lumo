use crate::wallet::error::{Result, WalletError};
use base64::{engine::general_purpose, Engine as _};

/// Simple encryption utility for mnemonic storage
/// TODO: Replace with platform keychain/keystore in production
pub struct MnemonicEncryption;

impl MnemonicEncryption {
    /// Encrypt a mnemonic for storage
    /// For now, just base64 encode (not secure!)
    /// TODO: Use proper encryption with platform keystore
    pub fn encrypt(mnemonic: &str) -> Result<String> {
        // WARNING: This is NOT secure encryption!
        // In production, use platform keychain (macOS Keychain, Windows Credential Store, etc.)
        let encoded = general_purpose::STANDARD.encode(mnemonic.as_bytes());
        Ok(format!("b64:{}", encoded))
    }

    /// Decrypt a mnemonic from storage
    pub fn decrypt(encrypted: &str) -> Result<String> {
        if let Some(data) = encrypted.strip_prefix("b64:") {
            let decoded = general_purpose::STANDARD
                .decode(data)
                .map_err(|e| WalletError::Generic(format!("Failed to decode mnemonic: {}", e)))?;

            let mnemonic = String::from_utf8(decoded)
                .map_err(|e| WalletError::Generic(format!("Invalid mnemonic encoding: {}", e)))?;

            Ok(mnemonic)
        } else {
            Err(WalletError::Generic(
                "Invalid encrypted mnemonic format".to_string(),
            ))
        }
    }

    /// Check if a string is encrypted
    pub fn is_encrypted(data: &str) -> bool {
        data.starts_with("b64:")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let encrypted = MnemonicEncryption::encrypt(original).unwrap();
        assert!(MnemonicEncryption::is_encrypted(&encrypted));

        let decrypted = MnemonicEncryption::decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
