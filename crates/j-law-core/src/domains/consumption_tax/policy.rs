use std::collections::HashSet;

use crate::domains::consumption_tax::context::ConsumptionTaxFlag;
use crate::types::rounding::RoundingStrategy;

/// 消費税計算のポリシーインターフェース。
///
/// 端数処理戦略や軽減税率の適用判定ロジックを差し替えられるようにする。
/// 通常は [`StandardConsumptionTaxPolicy`] を使う。
pub trait ConsumptionTaxPolicy: std::fmt::Debug {
    /// 軽減税率をフラグに基づいて適用するかどうかを判定する。
    fn should_apply_reduced_rate(&self, flags: &HashSet<ConsumptionTaxFlag>) -> bool;

    /// 税額計算に使う端数処理戦略。
    fn tax_rounding(&self) -> RoundingStrategy;
}

/// 消費税法の標準解釈に基づく消費税計算ポリシー。
///
/// # 法的根拠
/// 消費税法 第29条
/// 消費税法 第45条（端数処理: 1円未満切り捨て）
#[derive(Debug, Clone, Copy)]
pub struct StandardConsumptionTaxPolicy;

impl ConsumptionTaxPolicy for StandardConsumptionTaxPolicy {
    fn should_apply_reduced_rate(&self, flags: &HashSet<ConsumptionTaxFlag>) -> bool {
        flags.contains(&ConsumptionTaxFlag::ReducedRate)
    }

    fn tax_rounding(&self) -> RoundingStrategy {
        RoundingStrategy::Floor
    }
}
