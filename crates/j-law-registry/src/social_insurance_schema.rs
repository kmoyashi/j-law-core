use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SocialInsuranceFraction {
    pub numer: u64,
    pub denom: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrefectureHealthRateEntry {
    pub prefecture_code: u8,
    pub rate: SocialInsuranceFraction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialInsuranceParamsEntry {
    pub prefecture_health_rates: Vec<PrefectureHealthRateEntry>,
    pub care_rate: SocialInsuranceFraction,
    pub pension_rate: SocialInsuranceFraction,
    pub valid_standard_monthly_remunerations: Vec<u64>,
    pub pension_standard_monthly_remuneration_cap: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SocialInsuranceCitationEntry {
    pub law_name: String,
    pub article: u16,
    pub paragraph: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialInsuranceHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub citation: SocialInsuranceCitationEntry,
    pub params: SocialInsuranceParamsEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialInsuranceRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<SocialInsuranceHistoryEntry>,
}
