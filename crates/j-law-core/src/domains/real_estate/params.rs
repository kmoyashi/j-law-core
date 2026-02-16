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
