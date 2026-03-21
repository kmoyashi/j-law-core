use crate::error::{CalculationError, InputError, JLawError};
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
    /// `amount` の整数部（`whole`）のみに適用し、端数部（`numer`/`denom`）は無視する。
    ///
    /// # Important
    /// `amount.numer` / `amount.denom` が非ゼロ（端数あり）の場合、
    /// その端数部分は計算に含まれず**黙って切り捨てられます**。
    /// 端数を保持した状態でレートを適用したい場合は、
    /// 事前に `amount.finalize(rounding)` を呼び出して整数化してください。
    ///
    /// # エラー
    /// - `self.denom == 0` の場合は `InputError::ZeroDenominator` を返す。
    /// - `MultiplyOrder::MultiplyFirst` で `base * self.numer` がオーバーフローした場合は
    ///   `CalculationError::Overflow` を返す。
    pub fn apply(
        &self,
        amount: &IntermediateAmount,
        order: MultiplyOrder,
        rounding: RoundingStrategy,
    ) -> Result<IntermediateAmount, JLawError> {
        if self.denom == 0 {
            return Err(InputError::ZeroDenominator.into());
        }
        let base = amount.whole;
        let result_whole = match order {
            MultiplyOrder::MultiplyFirst => {
                let product = base.checked_mul(self.numer).ok_or_else(|| {
                    CalculationError::Overflow {
                        step: format!("rate_apply: {} * {}", base, self.numer),
                    }
                })?;
                rounding.apply_ratio(product, self.denom)?
            }
            MultiplyOrder::DivideFirst => rounding.apply_ratio(base / self.denom * self.numer, 1)?,
        };
        Ok(IntermediateAmount::from_exact(result_whole))
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
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
            crate::error::JLawError::Input(crate::error::InputError::ZeroDenominator)
        ));
    }

    #[test]
    fn overflow_returns_calculation_error() {
        // u64::MAX * 2 はオーバーフローする
        let rate = Rate {
            numer: 2,
            denom: 1,
        };
        let result = rate.apply(
            &exact(u64::MAX),
            MultiplyOrder::MultiplyFirst,
            RoundingStrategy::Floor,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::JLawError::Calculation(crate::error::CalculationError::Overflow { .. })
        ));
    }
}
