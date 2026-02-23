/// 印紙税額のブラケット（1区間分）。
///
/// 印紙税は税率ではなく固定額のため、`Rate` ではなく `tax_amount` を直接保持する。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書（不動産の譲渡に関する契約書）
#[derive(Debug, Clone)]
pub struct StampTaxBracket {
    /// ブラケットの表示名（例: "10万円超50万円以下"）。
    pub label: String,
    /// 契約金額の下限（円・この金額以上）。
    pub amount_from: u64,
    /// 契約金額の上限（円・この金額以下）。`None` は上限なし。
    pub amount_to_inclusive: Option<u64>,
    /// 本則税額（円）。
    pub tax_amount: u64,
    /// 軽減税額（円）。軽減措置の対象外の場合は `None`。
    pub reduced_tax_amount: Option<u64>,
}

/// 印紙税計算に使うパラメータセット。
///
/// `j-law-registry` がJSONからロードしてこの型に変換する。
///
/// # 法的根拠
/// 印紙税法 別表第一 / 租税特別措置法 第91条
#[derive(Debug, Clone)]
pub struct StampTaxParams {
    /// 税額ブラケットの一覧（契約金額の低い方から順に並ぶ）。
    pub brackets: Vec<StampTaxBracket>,
    /// 軽減措置の適用開始日（ISO 8601形式、例: "2014-04-01"）。
    pub reduced_rate_from: Option<String>,
    /// 軽減措置の適用終了日（ISO 8601形式、例: "2027-03-31"）。
    pub reduced_rate_until: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_construction() {
        let bracket = StampTaxBracket {
            label: "10万円超50万円以下".into(),
            amount_from: 100_001,
            amount_to_inclusive: Some(500_000),
            tax_amount: 400,
            reduced_tax_amount: Some(200),
        };
        assert_eq!(bracket.tax_amount, 400);
        assert_eq!(bracket.reduced_tax_amount, Some(200));
    }

    #[test]
    fn params_construction() {
        let params = StampTaxParams {
            brackets: vec![StampTaxBracket {
                label: "非課税".into(),
                amount_from: 0,
                amount_to_inclusive: Some(9_999),
                tax_amount: 0,
                reduced_tax_amount: None,
            }],
            reduced_rate_from: Some("2014-04-01".into()),
            reduced_rate_until: Some("2027-03-31".into()),
        };
        assert_eq!(params.brackets.len(), 1);
        assert!(params.reduced_rate_from.is_some());
    }
}
