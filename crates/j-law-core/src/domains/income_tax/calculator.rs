use std::collections::HashSet;

use crate::domains::income_tax::context::{IncomeTaxContext, IncomeTaxFlag};
use crate::domains::income_tax::params::IncomeTaxParams;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};
use crate::types::rate::{MultiplyOrder, Rate};

/// 所得税の計算ステップ（内訳明細用）。
///
/// 速算表の適用結果を記録する。
#[derive(Debug, Clone)]
pub struct IncomeTaxStep {
    /// ブラケットの表示名（例: "330万円超695万円以下"）。
    pub label: String,
    /// 課税所得金額（円）。
    pub taxable_income: u64,
    /// 適用された税率の分子。
    pub rate_numer: u64,
    /// 適用された税率の分母。
    pub rate_denom: u64,
    /// 速算表の控除額（円）。
    pub deduction: u64,
    /// このステップでの算出税額（課税所得 × 税率 - 控除額）。
    pub result: FinalAmount,
}

/// 所得税の計算結果。
///
/// # 法的根拠
/// 所得税法 第89条第1項（所得税の税率）
/// 復興財源確保法 第13条（復興特別所得税）
/// 国税通則法 第119条第1項（税額の端数処理）
#[derive(Debug, Clone)]
pub struct IncomeTaxResult {
    /// 基準所得税額（復興特別所得税を含まない）。
    pub base_tax: FinalAmount,
    /// 復興特別所得税額（適用されない場合は0円）。
    pub reconstruction_tax: FinalAmount,
    /// 申告納税額（基準所得税額 + 復興特別所得税額）。
    ///
    /// 100円未満切り捨て（国税通則法 第119条第1項）。
    pub total_tax: FinalAmount,
    /// 計算の内訳ステップ。
    pub breakdown: Vec<IncomeTaxStep>,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<IncomeTaxFlag>,
    /// 復興特別所得税が適用されたか。
    pub reconstruction_tax_applied: bool,
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率表・速算表方式）
/// 復興財源確保法 第13条（復興特別所得税 2.1%）
/// 国税通則法 第119条第1項（申告納税額の100円未満切り捨て）
///
/// # 計算手順
/// 1. 課税所得金額が該当するブラケットを特定する
/// 2. 速算表方式で基準所得税額を算出: 課税所得金額 × 税率 - 控除額
/// 3. 復興特別所得税が適用される場合: 基準所得税額 × 21/1000（1円未満切り捨て）
/// 4. 申告納税額 = 基準所得税額 + 復興特別所得税額（100円未満切り捨て）
pub fn calculate_income_tax(
    ctx: &IncomeTaxContext,
    params: &IncomeTaxParams,
) -> Result<IncomeTaxResult, JLawError> {
    let income = ctx.taxable_income;
    let tax_rounding = ctx.policy.tax_rounding();

    // --- 課税所得金額が0の場合 ---
    if income == 0 {
        return Ok(IncomeTaxResult {
            base_tax: FinalAmount::new(0),
            reconstruction_tax: FinalAmount::new(0),
            total_tax: FinalAmount::new(0),
            breakdown: vec![],
            applied_flags: ctx.flags.clone(),
            reconstruction_tax_applied: false,
        });
    }

    // --- 該当ブラケットの特定 ---
    let bracket = params
        .brackets
        .iter()
        .find(|b| {
            income >= b.income_from
                && match b.income_to_inclusive {
                    Some(to) => income <= to,
                    None => true,
                }
        })
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "課税所得金額 {}円 に対応する税率ブラケットが見つかりません",
                income
            ),
        })?;

    // --- 速算表方式による基準所得税額の計算 ---
    // 税額 = 課税所得金額 × 税率 - 控除額
    let rate = Rate {
        numer: bracket.rate_numer,
        denom: bracket.rate_denom,
    };
    let gross_tax = rate
        .apply(
            &IntermediateAmount::from_exact(income),
            MultiplyOrder::MultiplyFirst,
            tax_rounding,
        )?
        .finalize(tax_rounding)?;

    let base_tax_yen = gross_tax
        .as_yen()
        .checked_sub(bracket.deduction)
        .ok_or_else(|| CalculationError::Overflow {
            step: "base_tax_deduction".into(),
        })?;

    let base_tax = FinalAmount::new(base_tax_yen);

    let breakdown = vec![IncomeTaxStep {
        label: bracket.label.clone(),
        taxable_income: income,
        rate_numer: bracket.rate_numer,
        rate_denom: bracket.rate_denom,
        deduction: bracket.deduction,
        result: base_tax,
    }];

    // --- 復興特別所得税 ---
    let target_year = ctx.target_date.0;
    let apply_reconstruction = ctx
        .policy
        .should_apply_reconstruction_tax(target_year, &ctx.flags);

    let reconstruction_tax_yen = if apply_reconstruction {
        if let Some(rt_params) = &params.reconstruction_tax {
            let rt_rate = Rate {
                numer: rt_params.rate_numer,
                denom: rt_params.rate_denom,
            };
            let rt_rounding = ctx.policy.reconstruction_tax_rounding();
            rt_rate
                .apply(
                    &IntermediateAmount::from_exact(base_tax_yen),
                    MultiplyOrder::MultiplyFirst,
                    rt_rounding,
                )?
                .finalize(rt_rounding)?
                .as_yen()
        } else {
            return Err(CalculationError::PolicyNotApplicable {
                reason: "ApplyReconstructionTaxフラグが指定されましたが、パラメータセットに復興特別所得税データが含まれていません".into(),
            }
            .into());
        }
    } else {
        0
    };

    let reconstruction_tax = FinalAmount::new(reconstruction_tax_yen);

    // --- 申告納税額（100円未満切り捨て） ---
    // 国税通則法 第119条第1項
    let total_before_truncation = base_tax_yen
        .checked_add(reconstruction_tax_yen)
        .ok_or_else(|| CalculationError::Overflow {
            step: "total_tax".into(),
        })?;
    let total_tax = FinalAmount::new(truncate_below_100(total_before_truncation));

    Ok(IncomeTaxResult {
        base_tax,
        reconstruction_tax,
        total_tax,
        breakdown,
        applied_flags: ctx.flags.clone(),
        reconstruction_tax_applied: apply_reconstruction,
    })
}

/// 100円未満を切り捨てる（国税通則法 第119条第1項）。
fn truncate_below_100(amount: u64) -> u64 {
    amount / 100 * 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_below_100_works() {
        assert_eq!(truncate_below_100(0), 0);
        assert_eq!(truncate_below_100(99), 0);
        assert_eq!(truncate_below_100(100), 100);
        assert_eq!(truncate_below_100(199), 100);
        assert_eq!(truncate_below_100(584_522), 584_500);
    }
}
