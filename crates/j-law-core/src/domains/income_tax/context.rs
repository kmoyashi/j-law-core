use crate::domains::income_tax::policy::IncomeTaxPolicy;
use std::collections::HashSet;

/// 所得税計算に関わる法的フラグ。
///
/// WARNING: 各フラグの事実認定はライブラリの責任範囲外です。
/// 呼び出し元が正しく判断した上で指定してください。
///
/// # 法的根拠
/// 所得税法 各条項
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IncomeTaxFlag {
    /// 復興特別所得税を適用する。
    ///
    /// 適用要件: 2013年〜2037年の各年分の所得税。
    /// 法的根拠: 復興財源確保法 第13条
    ApplyReconstructionTax,
}

/// 所得税計算の入力コンテキスト。
///
/// # 法的根拠
/// 所得税法 第89条第1項
pub struct IncomeTaxContext {
    /// 課税所得金額（円）。
    ///
    /// 各種所得控除を適用した後の金額。
    /// 1,000円未満の端数は切り捨て済みであること。
    /// 法的根拠: 所得税法 第89条第1項
    pub taxable_income: u64,
    /// 対象年度 `(year, month, day)` — 確定申告の対象となる年の基準日。
    pub target_date: (u16, u8, u8),
    /// 適用する法的フラグの集合。
    pub flags: HashSet<IncomeTaxFlag>,
    /// 計算ポリシー（テスト・カスタム計算での差し替えを可能にする）。
    pub policy: Box<dyn IncomeTaxPolicy>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::income_tax::policy::StandardIncomeTaxPolicy;

    #[test]
    fn context_construction() {
        let mut flags = HashSet::new();
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);

        let ctx = IncomeTaxContext {
            taxable_income: 5_000_000,
            target_date: (2024, 1, 1),
            flags,
            policy: Box::new(StandardIncomeTaxPolicy),
        };
        assert_eq!(ctx.taxable_income, 5_000_000);
        assert!(ctx.flags.contains(&IncomeTaxFlag::ApplyReconstructionTax));
    }

    #[test]
    fn flag_equality() {
        assert_eq!(
            IncomeTaxFlag::ApplyReconstructionTax,
            IncomeTaxFlag::ApplyReconstructionTax,
        );
    }
}
