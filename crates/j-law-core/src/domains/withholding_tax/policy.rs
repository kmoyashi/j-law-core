use std::collections::HashSet;

use crate::domains::withholding_tax::context::{WithholdingTaxCategory, WithholdingTaxFlag};
use crate::types::rounding::RoundingStrategy;

/// 源泉徴収税額計算のポリシーインターフェース。
///
/// 通常は [`StandardWithholdingTaxPolicy`] を使う。
pub trait WithholdingTaxPolicy: std::fmt::Debug {
    /// 税額計算に使う端数処理戦略。
    fn tax_rounding(&self) -> RoundingStrategy;

    /// 応募作品等の入選賞金・謝金に対する非課税特例を適用するか判定する。
    fn should_apply_submission_prize_exemption(
        &self,
        category: WithholdingTaxCategory,
        taxable_payment_amount: u64,
        flags: &HashSet<WithholdingTaxFlag>,
        exemption_threshold: Option<u64>,
    ) -> bool;
}

/// 国税庁の標準解釈に基づく源泉徴収税額計算ポリシー。
///
/// # 法的根拠
/// 所得税法 第204条第1項
/// 東日本大震災からの復興のための施策を実施するために必要な財源の確保に関する特別措置法
#[derive(Debug, Clone, Copy)]
pub struct StandardWithholdingTaxPolicy;

impl WithholdingTaxPolicy for StandardWithholdingTaxPolicy {
    fn tax_rounding(&self) -> RoundingStrategy {
        RoundingStrategy::Floor
    }

    fn should_apply_submission_prize_exemption(
        &self,
        category: WithholdingTaxCategory,
        taxable_payment_amount: u64,
        flags: &HashSet<WithholdingTaxFlag>,
        exemption_threshold: Option<u64>,
    ) -> bool {
        category == WithholdingTaxCategory::ManuscriptAndLecture
            && flags.contains(&WithholdingTaxFlag::IsSubmissionPrize)
            && exemption_threshold
                .map(|threshold| taxable_payment_amount <= threshold)
                .unwrap_or(false)
    }
}
