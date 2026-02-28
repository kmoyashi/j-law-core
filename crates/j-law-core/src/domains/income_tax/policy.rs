use crate::domains::income_tax::context::IncomeTaxFlag;
use crate::types::rounding::RoundingStrategy;
use std::collections::HashSet;

/// 所得税計算のポリシーインターフェース。
///
/// 端数処理戦略や復興特別所得税の適用判定ロジックを差し替えられるようにする。
/// 通常は [`StandardIncomeTaxPolicy`] を使う。
pub trait IncomeTaxPolicy: std::fmt::Debug {
    /// 復興特別所得税を適用するかどうかを判定する。
    fn should_apply_reconstruction_tax(
        &self,
        target_year: u16,
        flags: &HashSet<IncomeTaxFlag>,
    ) -> bool;

    /// 速算表による税額計算に使う端数処理戦略。
    ///
    /// 所得税法上、課税所得金額 × 税率 の計算結果の端数処理。
    fn tax_rounding(&self) -> RoundingStrategy;

    /// 復興特別所得税額の端数処理戦略。
    ///
    /// 基準所得税額 × 2.1% の計算結果の端数処理。
    fn reconstruction_tax_rounding(&self) -> RoundingStrategy;
}

/// 国税庁の標準解釈に基づく所得税計算ポリシー。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率）
/// 復興財源確保法 第13条（復興特別所得税の税率）
/// 国税通則法 第119条第1項（税額の端数処理 — 100円未満切り捨て）
#[derive(Debug, Clone, Copy)]
pub struct StandardIncomeTaxPolicy;

impl IncomeTaxPolicy for StandardIncomeTaxPolicy {
    fn should_apply_reconstruction_tax(
        &self,
        target_year: u16,
        flags: &HashSet<IncomeTaxFlag>,
    ) -> bool {
        (2013..=2037).contains(&target_year)
            && flags.contains(&IncomeTaxFlag::ApplyReconstructionTax)
    }

    fn tax_rounding(&self) -> RoundingStrategy {
        // 所得税額は100円未満切り捨て（国税通則法 第119条第1項）。
        // ただし速算表の計算自体は整数演算で端数が出ないため、
        // 万一端数が発生した場合の安全策として Floor を指定する。
        RoundingStrategy::Floor
    }

    fn reconstruction_tax_rounding(&self) -> RoundingStrategy {
        // 復興特別所得税額は1円未満切り捨て。
        RoundingStrategy::Floor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> StandardIncomeTaxPolicy {
        StandardIncomeTaxPolicy
    }

    fn flags_with_reconstruction() -> HashSet<IncomeTaxFlag> {
        let mut flags = HashSet::new();
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
        flags
    }

    #[test]
    fn reconstruction_tax_within_period() {
        let flags = flags_with_reconstruction();
        assert!(policy().should_apply_reconstruction_tax(2024, &flags));
        assert!(policy().should_apply_reconstruction_tax(2013, &flags));
        assert!(policy().should_apply_reconstruction_tax(2037, &flags));
    }

    #[test]
    fn reconstruction_tax_outside_period() {
        let flags = flags_with_reconstruction();
        assert!(!policy().should_apply_reconstruction_tax(2012, &flags));
        assert!(!policy().should_apply_reconstruction_tax(2038, &flags));
    }

    #[test]
    fn reconstruction_tax_without_flag() {
        let flags = HashSet::new();
        assert!(!policy().should_apply_reconstruction_tax(2024, &flags));
    }

    #[test]
    fn rounding_strategies() {
        assert_eq!(policy().tax_rounding(), RoundingStrategy::Floor);
        assert_eq!(
            policy().reconstruction_tax_rounding(),
            RoundingStrategy::Floor
        );
    }
}
