use lumo_types::Amount;

#[derive(Debug, Clone)]
pub struct Balance(pub bdk_wallet::Balance);

impl Balance {
    pub fn confirmed(&self) -> Amount {
        self.0.confirmed.into()
    }

    pub fn spendable(&self) -> Amount {
        self.0.trusted_spendable().into()
    }
}
