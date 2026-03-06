/// 消費税率（分子・分母表現）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumptionTaxRate {
    pub numer: u64,
    pub denom: u64,
}

/// 消費税計算パラメータ。
///
/// # 法的根拠
/// 消費税法 第29条
///
/// # 税率の歴史
/// - 3%: 1989年4月1日〜1997年3月31日
/// - 5%: 1997年4月1日〜2014年3月31日
/// - 8%: 2014年4月1日〜2019年9月30日
/// - 10%（標準）/ 8%（軽減）: 2019年10月1日〜
#[derive(Debug, Clone)]
pub struct ConsumptionTaxParams {
    /// 標準税率。
    pub standard_rate: ConsumptionTaxRate,
    /// 軽減税率。2019年10月1日以降の飲食料品・新聞等に適用。それ以前は `None`。
    pub reduced_rate: Option<ConsumptionTaxRate>,
}
