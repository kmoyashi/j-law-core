/// 所得税の税率ブラケット（1段階分）。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率表）
#[derive(Debug, Clone)]
pub struct IncomeTaxBracket {
    /// ブラケットの表示名（例: "195万円以下"）。
    pub label: String,
    /// 課税所得金額の下限（円・この金額以上）。
    pub income_from: u64,
    /// 課税所得金額の上限（円・この金額以下）。`None` は上限なし。
    pub income_to_inclusive: Option<u64>,
    /// 税率の分子（例: 5% → numer=5）。
    pub rate_numer: u64,
    /// 税率の分母（例: 5% → denom=100）。
    pub rate_denom: u64,
    /// 控除額（円）。速算表の控除額。
    ///
    /// 速算表方式: 税額 = 課税所得金額 × 税率 - 控除額
    pub deduction: u64,
}

/// 復興特別所得税のパラメータ。
///
/// # 法的根拠
/// 東日本大震災からの復興のための施策を実施するために必要な財源の確保に関する特別措置法
/// 第13条（復興特別所得税の税率）
///
/// 2013年〜2037年の各年分の基準所得税額に対して課税される。
#[derive(Debug, Clone)]
pub struct ReconstructionTaxParams {
    /// 税率の分子（例: 2.1% → numer=21）。
    pub rate_numer: u64,
    /// 税率の分母（例: 2.1% → denom=1000）。
    pub rate_denom: u64,
    /// 適用開始年（含む）。
    pub effective_from_year: u16,
    /// 適用終了年（含む）。
    pub effective_to_year_inclusive: u16,
}

/// 所得税計算に使うパラメータセット。
///
/// `j-law-registry` がJSONからロードしてこの型に変換する。
/// `j-law-core` の計算ロジックはこの型のみに依存する。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率表）
#[derive(Debug, Clone)]
pub struct IncomeTaxParams {
    /// 税率ブラケットの一覧（課税所得金額の低い方から順に並ぶ）。
    pub brackets: Vec<IncomeTaxBracket>,
    /// 復興特別所得税のパラメータ（適用期間外の場合は `None`）。
    pub reconstruction_tax: Option<ReconstructionTaxParams>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_construction() {
        let bracket = IncomeTaxBracket {
            label: "195万円以下".into(),
            income_from: 0,
            income_to_inclusive: Some(1_950_000),
            rate_numer: 5,
            rate_denom: 100,
            deduction: 0,
        };
        assert_eq!(bracket.rate_numer, 5);
        assert_eq!(bracket.rate_denom, 100);
        assert_eq!(bracket.deduction, 0);
    }

    #[test]
    fn params_with_reconstruction_tax() {
        let params = IncomeTaxParams {
            brackets: vec![
                IncomeTaxBracket {
                    label: "195万円以下".into(),
                    income_from: 0,
                    income_to_inclusive: Some(1_950_000),
                    rate_numer: 5,
                    rate_denom: 100,
                    deduction: 0,
                },
            ],
            reconstruction_tax: Some(ReconstructionTaxParams {
                rate_numer: 21,
                rate_denom: 1000,
                effective_from_year: 2013,
                effective_to_year_inclusive: 2037,
            }),
        };
        assert_eq!(params.brackets.len(), 1);
        let rt = params.reconstruction_tax.as_ref().unwrap();
        assert_eq!(rt.rate_numer, 21);
        assert_eq!(rt.rate_denom, 1000);
    }

    #[test]
    fn params_without_reconstruction_tax() {
        let params = IncomeTaxParams {
            brackets: vec![],
            reconstruction_tax: None,
        };
        assert!(params.reconstruction_tax.is_none());
    }
}
