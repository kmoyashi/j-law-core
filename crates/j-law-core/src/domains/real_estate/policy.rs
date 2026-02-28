use crate::domains::real_estate::context::RealEstateFlag;
use crate::types::rounding::RoundingStrategy;
use std::collections::HashSet;

/// 媒介報酬計算のポリシーインターフェース。
///
/// 端数処理戦略や特例適用の判定ロジックを差し替えられるようにする。
/// 通常は [`StandardMliitPolicy`] を使う。
pub trait RealEstatePolicy: std::fmt::Debug {
    /// 低廉な空き家特例を適用するかどうかを判定する。
    fn should_apply_low_cost_special(&self, price: u64, flags: &HashSet<RealEstateFlag>) -> bool;

    /// 各ティアの計算に使う端数処理戦略。
    fn tier_rounding(&self) -> RoundingStrategy;

    /// 税額計算に使う端数処理戦略。
    fn tax_rounding(&self) -> RoundingStrategy;
}

/// 国土交通省の標準解釈に基づく媒介報酬計算ポリシー。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行）
#[derive(Debug, Clone, Copy)]
pub struct StandardMliitPolicy;

impl RealEstatePolicy for StandardMliitPolicy {
    fn should_apply_low_cost_special(&self, price: u64, flags: &HashSet<RealEstateFlag>) -> bool {
        price <= 8_000_000 && flags.contains(&RealEstateFlag::IsLowCostVacantHouse)
    }

    fn tier_rounding(&self) -> RoundingStrategy {
        RoundingStrategy::Floor
    }

    fn tax_rounding(&self) -> RoundingStrategy {
        RoundingStrategy::Floor
    }
}
