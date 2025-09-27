use bdk_esplora::{esplora_client, EsploraAsyncExt};
use bdk_wallet::chain::spk_client::{FullScanRequest, FullScanResponse};
use bdk_wallet::KeychainKind;

pub struct EsploraClient {
    client: esplora_client::AsyncClient,
}

impl EsploraClient {
    pub async fn new(url: &str) -> eyre::Result<Self> {
        let client = esplora_client::Builder::new(url).build_async()?;
        Ok(Self { client })
    }

    pub async fn full_scan(
        &self,
        request: FullScanRequest<KeychainKind>,
        stop_gap: usize,
    ) -> eyre::Result<FullScanResponse<KeychainKind>> {
        Ok(self.client.full_scan(request, stop_gap, 1).await?)
    }

    pub async fn broadcast_transaction(
        &self,
        transaction: &bitcoin::Transaction,
    ) -> eyre::Result<bitcoin::Txid> {
        self.client.broadcast(transaction).await?;
        Ok(transaction.compute_txid())
    }
}
