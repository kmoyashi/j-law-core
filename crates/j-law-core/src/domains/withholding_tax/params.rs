use crate::domains::withholding_tax::context::WithholdingTaxCategory;

/// 源泉徴収カテゴリごとの計算方式。
///
/// # 法的根拠
/// 所得税法 第204条第1項
#[derive(Debug, Clone)]
pub enum WithholdingTaxMethod {
    /// 100万円以下部分と超過部分で税率が異なる二段階計算。
    ///
    /// 税率は所得税率に付加税率を加えた合計税率（1円未満切り捨ては合計額で行う）。
    /// - 2013〜2026年: 所得税 + 復興特別所得税 2.1% → 10.21% / 20.42%
    /// - 2027〜2047年: 所得税 + 復興特別所得税 1.1% + 防衛特別所得税 1% → 10.21% / 20.42%
    /// - 2048年以後: 所得税 + 防衛特別所得税 1% → 10.1% / 20.2%
    ///
    /// 法的根拠: 復興財源確保法 第28条 / 防衛財源確保法 第5条の26・第5条の29第2項
    TwoTier {
        threshold: u64,
        base_rate_numer: u64,
        base_rate_denom: u64,
        excess_rate_numer: u64,
        excess_rate_denom: u64,
    },
}

/// 源泉徴収カテゴリ1件分のパラメータ。
///
/// # 法的根拠
/// 所得税法 第204条第1項
#[derive(Debug, Clone)]
pub struct WithholdingTaxCategoryParams {
    /// 対象カテゴリ。
    pub category: WithholdingTaxCategory,
    /// 表示用ラベル。
    pub label: String,
    /// 税額計算方式。
    pub method: WithholdingTaxMethod,
    /// 応募作品等の入選賞金・謝金の非課税しきい値。
    ///
    /// 該当しないカテゴリでは `None`。
    pub submission_prize_exemption_threshold: Option<u64>,
}

/// 源泉徴収税額計算に使うパラメータセット。
///
/// `j-law-registry` が JSON からロードしてこの型に変換する。
///
/// # 法的根拠
/// 所得税法 第204条第1項
#[derive(Debug, Clone)]
pub struct WithholdingTaxParams {
    /// カテゴリごとの計算パラメータ。
    pub categories: Vec<WithholdingTaxCategoryParams>,
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn params_construction() {
        let params = WithholdingTaxParams {
            categories: vec![WithholdingTaxCategoryParams {
                category: WithholdingTaxCategory::ManuscriptAndLecture,
                label: "原稿料・講演料等".into(),
                method: WithholdingTaxMethod::TwoTier {
                    threshold: 1_000_000,
                    base_rate_numer: 1021,
                    base_rate_denom: 10_000,
                    excess_rate_numer: 2042,
                    excess_rate_denom: 10_000,
                },
                submission_prize_exemption_threshold: Some(50_000),
            }],
        };

        assert_eq!(params.categories.len(), 1);
        assert_eq!(
            params.categories[0].submission_prize_exemption_threshold,
            Some(50_000)
        );
    }
}
