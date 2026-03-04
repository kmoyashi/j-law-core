use serde::Deserialize;

/// JSON の分数表現 `{ "numer": N, "denom": N }`。
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumptionTaxFraction {
    pub numer: u64,
    pub denom: u64,
}

/// 1世代の消費税パラメータ群。
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumptionTaxParamsEntry {
    pub standard_rate: ConsumptionTaxFraction,
    pub reduced_rate: Option<ConsumptionTaxFraction>,
}

/// 法令引用情報。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ConsumptionTaxCitationEntry {
    pub law_id: String,
    pub law_name: String,
    pub article: u16,
    pub paragraph: Option<u16>,
    pub ministry: String,
}

/// 1世代の履歴エントリ。
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumptionTaxHistoryEntry {
    /// 施行日 `"YYYY-MM-DD"`。
    pub effective_from: String,
    /// 廃止日 `"YYYY-MM-DD"`。現行版は `null`。
    pub effective_until: Option<String>,
    /// `"active"` または `"superseded"`。
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub citation: ConsumptionTaxCitationEntry,
    pub params: ConsumptionTaxParamsEntry,
}

/// `consumption_tax.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumptionTaxRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<ConsumptionTaxHistoryEntry>,
}
