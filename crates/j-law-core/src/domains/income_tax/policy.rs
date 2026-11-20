use crate::domains::income_tax::context::IncomeTaxFlag;
use crate::types::rounding::RoundingStrategy;
use std::collections::HashSet;

/// 所得税計算のポリシーインターフェース。
///
/// 端数処理戦略や付加税（復興特別所得税・防衛特別所得税）の適用判定ロジックを
/// 差し替えられるようにする。
/// 通常は [`StandardIncomeTaxPolicy`] を使う。
pub trait IncomeTaxPolicy: std::fmt::Debug {
    /// 付加税（復興特別所得税・防衛特別所得税）を適用するかどうかを判定する。
    ///
    /// 各付加税の課税期間・税率はパラメータ（registry）で判定するため、
    /// ここでは呼び出し元が付加税の適用を求めているかだけを返せばよい。
    fn should_apply_reconstruction_tax(
        &self,
        target_year: u16,
        flags: &HashSet<IncomeTaxFlag>,
    ) -> bool;

    /// 速算表による税額計算に使う端数処理戦略。
    ///
    /// 所得税法上、課税所得金額 × 税率 の計算結果の端数処理。
    fn tax_rounding(&self) -> RoundingStrategy;

    /// 付加税額の端数処理戦略。
    ///
    /// 基準所得税額 × 付加税率（合計額および防衛特別所得税額）の計算結果の端数処理。
    fn reconstruction_tax_rounding(&self) -> RoundingStrategy;
}

/// 国税庁の標準解釈に基づく所得税計算ポリシー。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率）
/// 復興財源確保法 第13条（復興特別所得税の税率）
/// 防衛財源確保法 第5条の9（防衛特別所得税の税率）
/// 国税通則法 第119条第1項（税額の端数処理 — 100円未満切り捨て）
#[derive(Debug, Clone, Copy)]
pub struct StandardIncomeTaxPolicy;

impl IncomeTaxPolicy for StandardIncomeTaxPolicy {
    fn should_apply_reconstruction_tax(
        &self,
        target_year: u16,
        flags: &HashSet<IncomeTaxFlag>,
    ) -> bool {
        // 課税期間はパラメータ側で判定する（法改正による期間延長に追随するため）。
        let _ = target_year;
        flags.contains(&IncomeTaxFlag::ApplyReconstructionTax)
    }

    fn tax_rounding(&self) -> RoundingStrategy {
        // 所得税額は100円未満切り捨て（国税通則法 第119条第1項）。
        // ただし速算表の計算自体は整数演算で端数が出ないため、
        // 万一端数が発生した場合の安全策として Floor を指定する。
        RoundingStrategy::Floor
    }

    fn reconstruction_tax_rounding(&self) -> RoundingStrategy {
        // 付加税額は1円未満切り捨て。
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
    fn special_taxes_follow_flag_regardless_of_year() {
        // 課税期間の判定はパラメータ側（calculator のテストで検証）。
        let flags = flags_with_reconstruction();
        for year in [2012, 2013, 2024, 2027, 2047, 2048] {
            assert!(policy().should_apply_reconstruction_tax(year, &flags));
        }
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
