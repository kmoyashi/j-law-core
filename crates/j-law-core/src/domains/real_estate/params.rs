/// 1ティアの計算パラメータ。
#[derive(Debug, Clone)]
pub struct TierParam {
    pub label: String,
    pub price_from: u64,
    /// `None` は上限なし。
    pub price_to_inclusive: Option<u64>,
    pub rate_numer: u64,
    pub rate_denom: u64,
}

/// 低廉な空き家特例パラメータ。
#[derive(Debug, Clone)]
pub struct LowCostSpecialParams {
    /// 特例対象となる売買価格の上限（この価格以下の場合に特例が適用される）。
    pub price_ceiling_inclusive: u64,
    /// 法令が定める報酬額の上限（税抜・円）。
    ///
    /// NOTE: フィールド名は法令上の「上限報酬額（ceiling）」を表すが、
    /// 計算ロジックでは「最低保証額（floor）」として機能する。
    /// 通常計算結果がこの値を下回る場合、この値まで引き上げられる。
    /// 参照: `calculator::calculate_brokerage_fee` のコメント。
    pub fee_ceiling_exclusive_tax: u64,
    /// `true` の場合、売主側の取引にのみ特例が適用される。
    ///
    /// 2018年1月1日〜2024年6月30日の特例は売主のみ対象（宅建業法改正告示・平成29年国土交通省告示第98号）。
    /// `false` の場合、売主・買主双方に適用される（2024年7月1日施行以降）。
    pub seller_only: bool,
}

/// 媒介報酬計算に使うパラメータセット。
///
/// `j-law-registry` がJSONからロードしてこの型に変換する。
/// `j-law-core` の計算ロジックはこの型のみに依存する。
#[derive(Debug, Clone)]
pub struct BrokerageFeeParams {
    pub tiers: Vec<TierParam>,
    pub tax_numer: u64,
    pub tax_denom: u64,
    pub low_cost_special: Option<LowCostSpecialParams>,
}
