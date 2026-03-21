use crate::domains::real_estate::context::RealEstateFlag;
use crate::types::rounding::RoundingStrategy;
use std::collections::HashSet;

/// 媒介報酬計算のポリシーインターフェース。
///
/// 端数処理戦略や特例適用の判定ロジックを差し替えられるようにする。
/// 通常は [`StandardMlitPolicy`] を使う。
///
/// # 設計上の注意
/// 価格の閾値チェック（`price_ceiling_inclusive`）は、
/// パラメータレジストリが持つ値を使うため `calculator` 側で行う。
/// このメソッドはフラグベースの判定のみを担う。
pub trait RealEstatePolicy: std::fmt::Debug {
    /// 低廉な空き家特例をフラグに基づいて適用するかどうかを判定する。
    ///
    /// 価格の閾値チェックは呼び出し元（`calculator`）がパラメータを用いて行う。
    fn should_apply_low_cost_special(&self, flags: &HashSet<RealEstateFlag>) -> bool;

    /// 各ティアの計算に使う端数処理戦略。
    fn tier_rounding(&self) -> RoundingStrategy;
}

/// 国土交通省の標準解釈に基づく媒介報酬計算ポリシー。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2018年1月1日施行・2024年7月1日改正）
#[derive(Debug, Clone, Copy)]
pub struct StandardMlitPolicy;

impl RealEstatePolicy for StandardMlitPolicy {
    fn should_apply_low_cost_special(&self, flags: &HashSet<RealEstateFlag>) -> bool {
        flags.contains(&RealEstateFlag::IsLowCostVacantHouse)
    }

    fn tier_rounding(&self) -> RoundingStrategy {
        RoundingStrategy::Floor
    }
}
