use lumo_types::Network;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)] 
pub struct FeeRateEstimation {
    pub fastestFee: f32,
    pub halfHourFee: f32,
    pub hourFee: f32,
    pub economyFee: f32,
    pub minimumFee: f32,
}

pub struct FeeRateOptions {
    pub fast: f32,
    pub medium: f32,
    pub slow: f32,
}

pub async fn fetch_fee_rates(
    network: Network,
) -> Result<FeeRateEstimation, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = match network {
        Network::Mainnet => "https://mempool.space/api/v1/fees/recommended",
        Network::Testnet => "https://mempool.space/testnet/api/v1/fees/recommended",
        Network::Signet => "https://mempool.space/signet/api/v1/fees/recommended",
        Network::Testnet4 => "https://mempool.space/testnet4/api/v1/fees/recommended",
        Network::Regtest => return Err("Regtest network doesn't support fee estimation".into()),
    };
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    Ok(serde_json::from_str(&body)?)
}

impl FeeRateOptions {
    pub fn new(fast: f32, medium: f32, slow: f32) -> Self {
        Self { fast, medium, slow }
    }

    pub fn from_estimation(estimation: &FeeRateEstimation) -> Self {
        Self {
            fast: estimation.fastestFee,
            medium: estimation.halfHourFee,
            slow: estimation.hourFee,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_testnet_fees() {
        let fees = fetch_fee_rates(Network::Testnet).await.unwrap();
        let options = FeeRateOptions::from_estimation(&fees);

        println!("Raw API response: {:?}", fees);
        println!("Fast: {} sat/vB", options.fast);
        println!("Medium: {} sat/vB", options.medium);
        println!("Slow: {} sat/vB", options.slow);

        // Basic sanity checks
        assert!(options.fast >= options.medium);
        assert!(options.medium >= options.slow);
        assert!(options.slow > 0.0);
    }
}
