use serde::Deserialize;

/// JSON の分数表現 `{ "numer": N, "denom": N }`。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxFraction {
    pub numer: u64,
    pub denom: u64,
}

/// 二段階税率方式の定義。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxTwoTierMethodEntry {
    pub kind: String,
    pub threshold: u64,
    pub base_rate: WithholdingTaxFraction,
    pub excess_rate: WithholdingTaxFraction,
}

/// カテゴリ単位のパラメータ。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxCategoryEntry {
    pub code: String,
    pub label: String,
    pub method: WithholdingTaxTwoTierMethodEntry,
    pub submission_prize_exemption_threshold: Option<u64>,
}

/// 1世代の計算パラメータ群。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxParamsEntry {
    pub categories: Vec<WithholdingTaxCategoryEntry>,
}

/// 法令引用情報。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WithholdingTaxCitationEntry {
    pub law_name: String,
    pub article: u16,
    pub paragraph: Option<u16>,
}

/// 1世代の履歴エントリ。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub citation: WithholdingTaxCitationEntry,
    pub params: WithholdingTaxParamsEntry,
}

/// `withholding_tax.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct WithholdingTaxRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<WithholdingTaxHistoryEntry>,
}
