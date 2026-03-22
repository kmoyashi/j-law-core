use std::collections::HashSet;

use crate::domains::consumption_tax::calculator::calculate_consumption_tax;
use crate::domains::consumption_tax::context::ConsumptionTaxContext;
use crate::domains::consumption_tax::policy::StandardConsumptionTaxPolicy;
use crate::domains::real_estate::context::{RealEstateContext, RealEstateFlag};
use crate::domains::real_estate::params::BrokerageFeeParams;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};
use crate::types::rate::{MultiplyOrder, Rate};

/// 1ティアの計算ステップ（内訳明細用）。
#[derive(Debug, Clone)]
pub struct BrokerageFeeStep {
    pub label: String,
    pub base_amount: u64,
    pub rate_numer: u64,
    pub rate_denom: u64,
    pub result: FinalAmount,
}

/// 媒介報酬の計算結果。
#[derive(Debug, Clone)]
pub struct BrokerageFeeResult {
    /// 税込合計額。
    pub total_with_tax: FinalAmount,
    /// 税抜合計額。
    pub total_without_tax: FinalAmount,
    /// 消費税額。
    pub tax_amount: FinalAmount,
    /// 各ティアの計算内訳。
    pub breakdown: Vec<BrokerageFeeStep>,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<RealEstateFlag>,
    /// 低廉な空き家特例が適用されたか。
    pub low_cost_special_applied: bool,
}

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行 / 2019年10月1日施行）
///
/// # 計算手順
/// 1. 各ティアの対象金額を求め、個別に切り捨てる
/// 2. 各ティアの結果を合算して税抜き合計を得る
/// 3. 低廉な空き家特例が適用される場合、通常計算が保証額を下回るなら保証額まで引き上げる
///    （NOTE: `.max()` による最低保証であり、`.min()` による上限キャップではない）
/// 4. 消費税ドメイン（消費税法第29条）に処理を委譲して税額・税込額を得る
pub fn calculate_brokerage_fee(
    ctx: &RealEstateContext,
    params: &BrokerageFeeParams,
) -> Result<BrokerageFeeResult, JLawError> {
    ctx.target_date.validate()?;

    let price = ctx.price;
    let tier_rounding = ctx.policy.tier_rounding();

    // --- ティア計算 ---
    let mut breakdown: Vec<BrokerageFeeStep> = Vec::new();
    let mut subtotal = 0u64;

    for tier in &params.tiers {
        let tier_base = compute_tier_base(price, tier.price_from, tier.price_to_inclusive);
        if tier_base == 0 {
            continue;
        }

        let rate = Rate::new(tier.rate_numer, tier.rate_denom).map_err(|_| {
            CalculationError::Overflow {
                step: tier.label.clone(),
            }
        })?;
        let amount = IntermediateAmount::from_exact(tier_base);
        let tier_result = rate.apply(&amount, MultiplyOrder::MultiplyFirst, tier_rounding)?;
        let tier_final = tier_result.finalize(tier_rounding)?;

        subtotal = subtotal.checked_add(tier_final.as_yen()).ok_or_else(|| {
            CalculationError::Overflow {
                step: tier.label.clone(),
            }
        })?;

        breakdown.push(BrokerageFeeStep {
            label: tier.label.clone(),
            base_amount: tier_base,
            rate_numer: tier.rate_numer,
            rate_denom: tier.rate_denom,
            result: tier_final,
        });
    }

    // --- 低廉な空き家特例 ---
    // 2018年1月1日施行（平成29年国土交通省告示第98号）。
    // 400万円以下（2024年7月以降は800万円以下）の低廉な空き家等については、
    // 通常計算が保証額を下回る場合でも保証額まで請求できる（最低保証額）。
    //
    // 適用条件（3つ全て満たす場合に適用）:
    //   1. 売買価格が params.low_cost_special.price_ceiling_inclusive 以下
    //   2. IsLowCostVacantHouse フラグが指定されている（ポリシーに委譲）
    //   3. seller_only == true の場合は IsSeller フラグも必要（2018〜2024年の制約）
    let mut low_cost_applied = false;
    if let Some(special) = &params.low_cost_special {
        let price_ok = price <= special.price_ceiling_inclusive;
        let flag_ok = ctx.policy.should_apply_low_cost_special(&ctx.flags);
        // seller_only の場合は IsSeller フラグが必須
        let party_ok = !special.seller_only || ctx.flags.contains(&RealEstateFlag::IsSeller);
        if price_ok && flag_ok && party_ok {
            // fee_ceiling_exclusive_tax は法令上の「上限報酬額」だが、
            // 特例適用時は「最低保証額」として機能する（通常計算が下回れば引き上げ）。
            subtotal = subtotal.max(special.fee_ceiling_exclusive_tax);
            low_cost_applied = true;
        }
    }

    let total_without_tax = FinalAmount::new(subtotal);

    // --- 消費税（消費税ドメインに委譲）---
    let tax_ctx = ConsumptionTaxContext {
        amount: subtotal,
        target_date: ctx.target_date,
        flags: HashSet::new(), // 不動産仲介報酬は標準税率のみ適用
        policy: Box::new(StandardConsumptionTaxPolicy),
    };
    let tax_result = calculate_consumption_tax(&tax_ctx, &params.consumption_tax)?;

    Ok(BrokerageFeeResult {
        total_with_tax: tax_result.amount_with_tax,
        total_without_tax,
        tax_amount: tax_result.tax_amount,
        breakdown,
        applied_flags: ctx.flags.clone(),
        low_cost_special_applied: low_cost_applied,
    })
}

/// ティアに対応する課税対象金額（price のうちこのティア範囲に収まる部分）を返す。
pub(crate) fn compute_tier_base(price: u64, from: u64, to_inclusive: Option<u64>) -> u64 {
    if price < from {
        return 0;
    }
    let capped = match to_inclusive {
        Some(ceiling) => price.min(ceiling),
        None => price,
    };
    if from == 0 {
        capped
    } else {
        // `price_from` は前ティア上限の翌円を表す（例: tier2 は 2_000_001 から始まる）。
        // そのため、このティアの課税対象金額は `capped - (from - 1)` で求まる。
        capped - (from - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_base_tier1_under() {
        assert_eq!(compute_tier_base(1_000_000, 0, Some(2_000_000)), 1_000_000);
    }

    #[test]
    fn tier_base_tier1_at_boundary() {
        assert_eq!(compute_tier_base(2_000_000, 0, Some(2_000_000)), 2_000_000);
    }

    #[test]
    fn tier_base_tier1_over() {
        assert_eq!(compute_tier_base(5_000_000, 0, Some(2_000_000)), 2_000_000);
    }

    #[test]
    fn tier_base_tier2_base() {
        // 5,000,000円 の tier2（from=2,000,001, to=4,000,000）
        // capped = min(5M, 4M) = 4M
        // base = 4M - (2_000_001 - 1) = 4M - 2M = 2,000,000
        assert_eq!(
            compute_tier_base(5_000_000, 2_000_001, Some(4_000_000)),
            2_000_000
        );
    }

    #[test]
    fn tier_base_price_below_from() {
        assert_eq!(compute_tier_base(1_000_000, 2_000_001, Some(4_000_000)), 0);
    }

    #[test]
    fn tier_base_no_ceiling() {
        // 5,000,000円 の tier3（from=4,000,001, 上限なし）
        // base = 5M - (4_000_001 - 1) = 5M - 4M = 1,000,000
        assert_eq!(compute_tier_base(5_000_000, 4_000_001, None), 1_000_000);
    }
}
