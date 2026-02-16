/// 端数処理戦略。
///
/// 法令計算では端数処理の根拠を明示する必要があるため、
/// `f64::floor` や `f64::round` を使わず、この型で整数演算する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingStrategy {
    /// 切り捨て（床関数）。法令計算で最も多く使われる。
    Floor,
    /// 四捨五入。
    HalfUp,
    /// 切り上げ（天井関数）。
    Ceil,
}

impl RoundingStrategy {
    /// `numer / denom` を整数で丸める（整数演算のみ・float不使用）。
    ///
    /// # パニック
    /// `denom == 0` の場合はパニックする（呼び出し元で保証すること）。
    pub(crate) fn apply_ratio(self, numer: u64, denom: u64) -> u64 {
        assert!(denom != 0, "denom must not be zero");
        match self {
            RoundingStrategy::Floor => numer / denom,
            RoundingStrategy::Ceil => (numer + denom - 1) / denom,
            RoundingStrategy::HalfUp => {
                // numer / denom を四捨五入: (numer * 2 + denom) / (denom * 2)
                // オーバーフロー対策: numer + denom/2 が安全な範囲かチェック不要
                // (denom は通常小さい値のため問題ない)
                (numer + denom / 2) / denom
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_truncates() {
        assert_eq!(RoundingStrategy::Floor.apply_ratio(10, 3), 3);
        assert_eq!(RoundingStrategy::Floor.apply_ratio(9, 3), 3);
        assert_eq!(RoundingStrategy::Floor.apply_ratio(0, 5), 0);
    }

    #[test]
    fn ceil_rounds_up() {
        assert_eq!(RoundingStrategy::Ceil.apply_ratio(10, 3), 4);
        assert_eq!(RoundingStrategy::Ceil.apply_ratio(9, 3), 3);
        assert_eq!(RoundingStrategy::Ceil.apply_ratio(1, 5), 1);
    }

    #[test]
    fn half_up_rounds() {
        // 5 / 2 = 2.5 → 3
        assert_eq!(RoundingStrategy::HalfUp.apply_ratio(5, 2), 3);
        // 4 / 2 = 2.0 → 2
        assert_eq!(RoundingStrategy::HalfUp.apply_ratio(4, 2), 2);
        // 7 / 3 = 2.333... → 2
        assert_eq!(RoundingStrategy::HalfUp.apply_ratio(7, 3), 2);
        // 8 / 3 = 2.666... → 3
        assert_eq!(RoundingStrategy::HalfUp.apply_ratio(8, 3), 3);
    }
}
