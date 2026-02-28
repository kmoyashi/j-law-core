use crate::types::amount::IntermediateAmount;
use crate::types::rounding::RoundingStrategy;

/// 乗算順序の指定。
///
/// 端数処理が絡む計算では乗算と除算の順序で結果が変わる場合があるため、
/// 明示的に指定できるようにしている。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplyOrder {
    /// 先に分子を掛けてから分母で割る（精度優先）。
    MultiplyFirst,
    /// 先に分母で割ってから分子を掛ける（オーバーフロー回避優先）。
    DivideFirst,
}

/// 分数で表された比率（例: 5/100 = 5%）。
///
/// float を使わず整数分数で保持することで、法令計算に必要な
/// 再現性のある端数処理を保証する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rate {
    /// 分子。
    pub numer: u64,
    /// 分母。0 は不正値。
    pub denom: u64,
}

impl Rate {
    /// `amount` にこのレートを適用して新しい `IntermediateAmount` を返す。
    ///
    /// `amount` の整数部のみに適用し、端数部は無視する（呼び出し前に finalize 推奨）。
    ///
    /// # エラー
    /// `self.denom == 0` の場合は `InputError::ZeroDenominator` を返す。
    pub fn apply(
        &self,
        amount: &IntermediateAmount,
        order: MultiplyOrder,
        rounding: RoundingStrategy,
    ) -> Result<IntermediateAmount, crate::error::InputError> {
        let base = amount.whole;
        let result_whole = match order {
            MultiplyOrder::MultiplyFirst => rounding.apply_ratio(base * self.numer, self.denom)?,
            MultiplyOrder::DivideFirst => {
                rounding.apply_ratio(base / self.denom * self.numer, 1)?
            }
        };
        Ok(IntermediateAmount::from_exact(result_whole))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(yen: u64) -> IntermediateAmount {
        IntermediateAmount::from_exact(yen)
    }

    #[test]
    fn rate_5_percent_multiply_first() {
        // 2_000_000 × 5/100 = 100_000
        let rate = Rate {
            numer: 5,
            denom: 100,
        };
        let result = rate
            .apply(
                &exact(2_000_000),
                MultiplyOrder::MultiplyFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        assert_eq!(
            result.finalize(RoundingStrategy::Floor).unwrap().as_yen(),
            100_000
        );
    }

    #[test]
    fn rate_4_percent_tier2() {
        // (4_000_000 - 2_000_000) × 4/100 = 80_000
        let rate = Rate {
            numer: 4,
            denom: 100,
        };
        let result = rate
            .apply(
                &exact(2_000_000),
                MultiplyOrder::MultiplyFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        assert_eq!(
            result.finalize(RoundingStrategy::Floor).unwrap().as_yen(),
            80_000
        );
    }

    #[test]
    fn rate_3_percent_tier3() {
        // (5_000_000 - 4_000_000) × 3/100 = 30_000
        let rate = Rate {
            numer: 3,
            denom: 100,
        };
        let result = rate
            .apply(
                &exact(1_000_000),
                MultiplyOrder::MultiplyFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        assert_eq!(
            result.finalize(RoundingStrategy::Floor).unwrap().as_yen(),
            30_000
        );
    }

    #[test]
    fn multiply_first_vs_divide_first_differ() {
        // 10 × 1/3:
        // MultiplyFirst: floor(10/3) = 3
        // DivideFirst:   floor(10/3) * 1 = 3  ← 同じ
        // 差が出るケース: 7 × 3/4
        // MultiplyFirst: floor(21/4) = 5
        // DivideFirst:   floor(7/4) * 3 = 1 * 3 = 3
        let rate = Rate { numer: 3, denom: 4 };
        let mf = rate
            .apply(
                &exact(7),
                MultiplyOrder::MultiplyFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        let df = rate
            .apply(
                &exact(7),
                MultiplyOrder::DivideFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        assert_eq!(mf.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 5);
        assert_eq!(df.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 3);
    }

    #[test]
    fn tax_10_percent() {
        // 210_000 × 10/100 = 21_000
        let rate = Rate {
            numer: 10,
            denom: 100,
        };
        let result = rate
            .apply(
                &exact(210_000),
                MultiplyOrder::MultiplyFirst,
                RoundingStrategy::Floor,
            )
            .unwrap();
        assert_eq!(
            result.finalize(RoundingStrategy::Floor).unwrap().as_yen(),
            21_000
        );
    }

    #[test]
    fn zero_denominator_returns_error() {
        let rate = Rate { numer: 1, denom: 0 };
        let result = rate.apply(
            &exact(100),
            MultiplyOrder::MultiplyFirst,
            RoundingStrategy::Floor,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::InputError::ZeroDenominator
        ));
    }
}
