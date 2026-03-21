use crate::error::InputError;
use crate::types::rounding::RoundingStrategy;

/// 計算の最終結果を表す金額型（円単位・整数）。
///
/// 税込合計・税抜合計・税額など、ユーザーに返す確定値にのみ使う。
/// 計算途中では [`IntermediateAmount`] を使うこと。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalAmount(u64);

impl FinalAmount {
    /// 円単位の値から `FinalAmount` を作る。
    pub fn new(yen: u64) -> Self {
        Self(yen)
    }

    /// 円単位の値を返す。
    pub fn as_yen(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for FinalAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}円", self.0)
    }
}

/// 計算途中の金額を分数で表す型。
///
/// `whole + numer/denom` を表す。例えば `100 + 1/3` は
/// `IntermediateAmount { whole: 100, numer: 1, denom: 3 }`。
///
/// 端数処理が必要な場面では [`IntermediateAmount::finalize`] を呼ぶこと。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntermediateAmount {
    /// 整数部分（円）。
    pub(crate) whole: u64,
    /// 分子。
    pub(crate) numer: u64,
    /// 分母。`0` は不正値であり、コンストラクタで拒否される。
    pub(crate) denom: u64,
}

impl IntermediateAmount {
    /// 整数値（円）から `IntermediateAmount` を作る（端数なし）。
    pub fn from_exact(yen: u64) -> Self {
        Self {
            whole: yen,
            numer: 0,
            denom: 1,
        }
    }

    /// 分数形式で作る。`denom == 0` の場合はエラーを返す。
    pub fn try_new(whole: u64, numer: u64, denom: u64) -> Result<Self, InputError> {
        if denom == 0 {
            return Err(InputError::ZeroDenominator);
        }
        Ok(Self {
            whole,
            numer,
            denom,
        })
    }

    /// 端数処理して [`FinalAmount`] に変換する。
    ///
    /// # エラー
    /// `self.denom == 0` の場合は `InputError::ZeroDenominator` を返す。
    pub fn finalize(&self, rounding: RoundingStrategy) -> Result<FinalAmount, InputError> {
        let frac = if self.numer == 0 {
            0
        } else {
            rounding.apply_ratio(self.numer, self.denom)?
        };
        Ok(FinalAmount::new(self.whole + frac))
    }

    /// 整数部分（円）を返す。
    pub fn whole(&self) -> u64 {
        self.whole
    }

    /// 分子を返す。
    pub fn numer(&self) -> u64 {
        self.numer
    }

    /// 分母を返す。
    pub fn denom(&self) -> u64 {
        self.denom
    }
}

/// `IntermediateAmount` 同士の加算を標準トレイトで実装。
impl std::ops::Add<&IntermediateAmount> for IntermediateAmount {
    type Output = IntermediateAmount;

    /// 加算（整数部分同士を加える）。
    ///
    /// 通分計算 `(a.numer * b.denom + b.numer * a.denom) / (a.denom * b.denom)` において
    /// 乗算・加算がオーバーフローした場合は `whole = u64::MAX` をセットして返す。
    /// 法令計算で扱う金額の範囲では通常発生しないが、
    /// 異常入力に対して silently panic しないための安全対策。
    fn add(self, other: &IntermediateAmount) -> IntermediateAmount {
        // 通分: frac 部は (a.numer * b.denom + b.numer * a.denom) / (a.denom * b.denom)
        let Some(new_denom) = self.denom.checked_mul(other.denom) else {
            return IntermediateAmount {
                whole: u64::MAX,
                numer: 0,
                denom: 1,
            };
        };
        let Some(lhs) = self.numer.checked_mul(other.denom) else {
            return IntermediateAmount {
                whole: u64::MAX,
                numer: 0,
                denom: 1,
            };
        };
        let Some(rhs) = other.numer.checked_mul(self.denom) else {
            return IntermediateAmount {
                whole: u64::MAX,
                numer: 0,
                denom: 1,
            };
        };
        let Some(new_numer) = lhs.checked_add(rhs) else {
            return IntermediateAmount {
                whole: u64::MAX,
                numer: 0,
                denom: 1,
            };
        };
        // 整数部を繰り上げながら正規化
        let carry = new_numer / new_denom;
        let rem = new_numer % new_denom;
        let Some(whole) = self
            .whole
            .checked_add(other.whole)
            .and_then(|w| w.checked_add(carry))
        else {
            return IntermediateAmount {
                whole: u64::MAX,
                numer: 0,
                denom: 1,
            };
        };
        IntermediateAmount {
            whole,
            numer: rem,
            denom: new_denom,
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::ops::Add;

    #[test]
    fn final_amount_roundtrip() {
        let a = FinalAmount::new(210_000);
        assert_eq!(a.as_yen(), 210_000);
    }

    #[test]
    fn from_exact_has_no_fraction() {
        let a = IntermediateAmount::from_exact(100);
        assert_eq!(a.whole(), 100);
        assert_eq!(a.numer(), 0);
    }

    #[test]
    fn try_new_rejects_zero_denom() {
        let result = IntermediateAmount::try_new(100, 1, 0);
        assert!(matches!(result, Err(InputError::ZeroDenominator)));
    }

    #[test]
    fn finalize_floor() {
        // 100 + 1/3 → Floor → 100
        let a = IntermediateAmount::try_new(100, 1, 3).unwrap();
        assert_eq!(a.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 100);
    }

    #[test]
    fn finalize_ceil() {
        // 100 + 1/3 → Ceil → 101
        let a = IntermediateAmount::try_new(100, 1, 3).unwrap();
        assert_eq!(a.finalize(RoundingStrategy::Ceil).unwrap().as_yen(), 101);
    }

    #[test]
    fn finalize_half_up() {
        // 100 + 1/2 = 100.5 → HalfUp → 101
        let a = IntermediateAmount::try_new(100, 1, 2).unwrap();
        assert_eq!(a.finalize(RoundingStrategy::HalfUp).unwrap().as_yen(), 101);

        // 100 + 1/3 = 100.333 → HalfUp → 100
        let b = IntermediateAmount::try_new(100, 1, 3).unwrap();
        assert_eq!(b.finalize(RoundingStrategy::HalfUp).unwrap().as_yen(), 100);
    }

    #[test]
    fn finalize_no_fraction() {
        let a = IntermediateAmount::from_exact(5_000);
        assert_eq!(a.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 5_000);
    }

    #[test]
    fn add_two_exact() {
        let a = IntermediateAmount::from_exact(100);
        let b = IntermediateAmount::from_exact(200);
        let c = a.add(&b);
        assert_eq!(c.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 300);
    }

    #[test]
    fn add_carries_fraction() {
        // 0 + 2/3  +  0 + 2/3  = 4/3 → carry 1, rem 1/3
        let a = IntermediateAmount::try_new(0, 2, 3).unwrap();
        let b = IntermediateAmount::try_new(0, 2, 3).unwrap();
        let c = a.add(&b);
        assert_eq!(c.whole(), 1);
        // finalize Floor → 1
        assert_eq!(c.finalize(RoundingStrategy::Floor).unwrap().as_yen(), 1);
    }
}
