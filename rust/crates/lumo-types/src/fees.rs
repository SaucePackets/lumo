use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FeeRate(bitcoin::FeeRate);

impl FeeRate {
    // Create fee rate from sat/vB
    pub fn from_sat_per_vb(sat_per_vb: f32) -> Self {
        let sat_per_kwu = (sat_per_vb * 250.0).ceil() as u64;
        let fee_rate = bitcoin::FeeRate::from_sat_per_kwu(sat_per_kwu);
        Self(fee_rate)
    }

    // Get fee rate as sat/vB
    pub fn as_sat_per_vb(&self) -> f32 {
        self.0.to_sat_per_kwu() as f32 / 250.0
    }
}

impl Into<bitcoin::FeeRate> for FeeRate {
    fn into(self) -> bitcoin::FeeRate {
        self.0
    }
}

impl std::fmt::Display for FeeRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} sat/vB", self.as_sat_per_vb())
    }
}
