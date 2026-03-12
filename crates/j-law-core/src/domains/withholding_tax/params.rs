use crate::domains::withholding_tax::context::WithholdingTaxCategory;

/// 源泉徴収カテゴリごとの計算方式。
///
/// # 法的根拠
/// 所得税法 第204条第1項
#[derive(Debug, Clone)]
pub enum WithholdingTaxMethod {
    /// 100万円以下部分と超過部分で税率が異なる二段階計算。
    ///
    /// 令和8年（2026年）3月11日時点の国税庁資料では
    /// 10.21% / 20.42% の二段階税率で運用されている。
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
