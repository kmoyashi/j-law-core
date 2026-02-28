use serde::Deserialize;

/// 印紙税額のブラケット（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxBracketEntry {
    pub label: String,
    pub amount_from: u64,
    pub amount_to_inclusive: Option<u64>,
    pub tax_amount: u64,
    pub reduced_tax_amount: Option<u64>,
}

/// 印紙税の計算パラメータ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxParamsEntry {
    pub brackets: Vec<StampTaxBracketEntry>,
    pub reduced_rate_from: Option<String>,
    pub reduced_rate_until: Option<String>,
}

/// 印紙税の履歴エントリ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub params: StampTaxParamsEntry,
}

/// `stamp_tax.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<StampTaxHistoryEntry>,
}
