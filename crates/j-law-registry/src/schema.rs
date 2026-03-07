use serde::Deserialize;

/// JSON の分数表現 `{ "numer": N, "denom": N }`。
#[derive(Debug, Clone, Deserialize)]
pub struct Fraction {
    pub numer: u64,
    pub denom: u64,
}

/// 1ティアの定義。
#[derive(Debug, Clone, Deserialize)]
pub struct TierParam {
    pub label: String,
    pub price_from: u64,
    /// `null` は「上限なし」を意味する。
    pub price_to_inclusive: Option<u64>,
    pub rate: Fraction,
}

/// 低廉な空き家特例パラメータ（2018年1月施行・2024年7月改正）。
#[derive(Debug, Clone, Deserialize)]
pub struct LowCostSpecialParam {
    /// 特例が適用される売買価格の上限（以下）。
    pub price_ceiling_inclusive: u64,
    /// 税抜き報酬額の上限（最低保証額として機能）。
    pub fee_ceiling_exclusive_tax: u64,
    /// `true` の場合、売主側の取引にのみ特例が適用される。
    ///
    /// 2018年1月1日〜2024年6月30日の特例は売主のみ対象（平成29年国土交通省告示第98号）。
    /// `false` の場合、売主・買主双方に適用される（2024年7月1日施行以降）。
    pub seller_only: bool,
}

/// 1世代の計算パラメータ群。
#[derive(Debug, Clone, Deserialize)]
pub struct ParamsEntry {
    pub tiers: Vec<TierParam>,
    pub low_cost_special: Option<LowCostSpecialParam>,
}

/// 法令引用情報。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CitationEntry {
    pub law_id: String,
    pub law_name: String,
    pub article: u16,
    pub paragraph: Option<u16>,
    pub ministry: String,
}

/// 1世代の履歴エントリ。
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    /// 施行日 `"YYYY-MM-DD"`。
    pub effective_from: String,
    /// 廃止日 `"YYYY-MM-DD"`。現行版は `null`。
    pub effective_until: Option<String>,
    /// `"active"` または `"superseded"`。
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub citation: CitationEntry,
    pub params: ParamsEntry,
}

/// `brokerage_fee.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct BrokerageFeeRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<HistoryEntry>,
}
