use std::collections::HashSet;

use crate::domains::withholding_tax::context::{
    WithholdingTaxCategory, WithholdingTaxContext, WithholdingTaxFlag,
};
use crate::domains::withholding_tax::params::{WithholdingTaxMethod, WithholdingTaxParams};
use crate::domains::withholding_tax::policy::WithholdingTaxPolicy;
use crate::error::{CalculationError, InputError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};
use crate::types::rate::{MultiplyOrder, Rate};

/// 源泉徴収税額の計算内訳1行。
#[derive(Debug, Clone)]
pub struct WithholdingTaxStep {
    /// 内訳ラベル。
    pub label: String,
    /// 当該行の対象金額（円）。
    pub base_amount: u64,
    /// 適用税率の分子。
    pub rate_numer: u64,
    /// 適用税率の分母。
    pub rate_denom: u64,
    /// 当該行の税額（円）。
    pub result: FinalAmount,
}

/// 源泉徴収税額の計算結果。
///
/// # 法的根拠
/// 所得税法 第204条第1項
/// 東日本大震災からの復興のための施策を実施するために必要な財源の確保に関する特別措置法
#[derive(Debug, Clone)]
pub struct WithholdingTaxResult {
    /// 支払総額（円）。
    pub gross_payment_amount: FinalAmount,
    /// 源泉徴収税額の計算対象額（円）。
    pub taxable_payment_amount: FinalAmount,
    /// 源泉徴収税額（円）。
    pub tax_amount: FinalAmount,
    /// 支払総額から源泉徴収税額を控除した後の金額（円）。
    pub net_payment_amount: FinalAmount,
    /// 適用されたカテゴリ。
    pub category: WithholdingTaxCategory,
    /// カテゴリ表示名。
    pub category_label: String,
    /// 応募作品等の入選賞金・謝金の非課税特例を適用したか。
    pub submission_prize_exempted: bool,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<WithholdingTaxFlag>,
    /// 計算内訳。
    pub breakdown: Vec<WithholdingTaxStep>,
}

/// 所得税法第204条第1項に基づく報酬・料金等の源泉徴収税額を計算する。
///
/// # 法的根拠
/// 所得税法 第204条第1項（報酬・料金等の源泉徴収）
/// 国税庁タックスアンサー No.2795 / No.2798 / No.2810
///
/// # 計算手順
/// 1. 請求書等で区分表示された消費税額を支払総額から控除し、課税対象額を求める
/// 2. カテゴリに対応する計算方式を決定する
/// 3. 応募作品等の入選賞金・謝金の非課税特例があれば税額を 0 円とする
/// 4. 100万円以下部分と超過部分にそれぞれ税率を適用し、1円未満切り捨てで合算する
pub fn calculate_withholding_tax(
    ctx: &WithholdingTaxContext,
    params: &WithholdingTaxParams,
) -> Result<WithholdingTaxResult, JLawError> {
    let taxable_payment_amount = ctx
        .payment_amount
        .checked_sub(ctx.separated_consumption_tax_amount)
        .ok_or_else(|| InputError::InvalidWithholdingInput {
            field: "separated_consumption_tax_amount".into(),
            reason: "消費税額が支払総額を超えています".into(),
        })?;

    let category_params = params
        .categories
        .iter()
        .find(|category| category.category == ctx.category)
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "カテゴリ {} に対応する源泉徴収パラメータが見つかりません",
                ctx.category
            ),
        })?;

    let exempted = ctx.policy.should_apply_submission_prize_exemption(
        ctx.category,
        taxable_payment_amount,
        &ctx.flags,
        category_params.submission_prize_exemption_threshold,
    );

    if exempted || taxable_payment_amount == 0 {
        return Ok(WithholdingTaxResult {
            gross_payment_amount: FinalAmount::new(ctx.payment_amount),
            taxable_payment_amount: FinalAmount::new(taxable_payment_amount),
            tax_amount: FinalAmount::new(0),
            net_payment_amount: FinalAmount::new(ctx.payment_amount),
            category: ctx.category,
            category_label: category_params.label.clone(),
            submission_prize_exempted: exempted,
            applied_flags: ctx.flags.clone(),
            breakdown: vec![],
        });
    }

    let (tax_amount, breakdown) = match &category_params.method {
        WithholdingTaxMethod::TwoTier {
            threshold,
            base_rate_numer,
            base_rate_denom,
            excess_rate_numer,
            excess_rate_denom,
        } => calculate_two_tier(
            taxable_payment_amount,
            *threshold,
            *base_rate_numer,
            *base_rate_denom,
            *excess_rate_numer,
            *excess_rate_denom,
            ctx.policy.as_ref(),
        )?,
    };

    let net_payment_amount = ctx
        .payment_amount
        .checked_sub(tax_amount.as_yen())
        .ok_or_else(|| CalculationError::Overflow {
            step: "net_payment_amount".into(),
        })?;

    Ok(WithholdingTaxResult {
        gross_payment_amount: FinalAmount::new(ctx.payment_amount),
        taxable_payment_amount: FinalAmount::new(taxable_payment_amount),
        tax_amount,
        net_payment_amount: FinalAmount::new(net_payment_amount),
        category: ctx.category,
        category_label: category_params.label.clone(),
        submission_prize_exempted: false,
        applied_flags: ctx.flags.clone(),
        breakdown,
    })
}

fn calculate_two_tier(
    amount: u64,
    threshold: u64,
    base_rate_numer: u64,
    base_rate_denom: u64,
    excess_rate_numer: u64,
    excess_rate_denom: u64,
    policy: &dyn WithholdingTaxPolicy,
) -> Result<(FinalAmount, Vec<WithholdingTaxStep>), JLawError> {
    let base_amount = amount.min(threshold);
    let excess_amount = amount.saturating_sub(threshold);

    let mut breakdown = Vec::with_capacity(if excess_amount > 0 { 2 } else { 1 });
    let base_tax = apply_rate(base_amount, base_rate_numer, base_rate_denom, policy)?;
    breakdown.push(WithholdingTaxStep {
        label: format!("{}円以下の部分", threshold),
        base_amount,
        rate_numer: base_rate_numer,
        rate_denom: base_rate_denom,
        result: base_tax,
    });

    let mut total_tax = base_tax.as_yen();

    if excess_amount > 0 {
        let excess_tax = apply_rate(excess_amount, excess_rate_numer, excess_rate_denom, policy)?;
        total_tax = total_tax.checked_add(excess_tax.as_yen()).ok_or_else(|| {
            CalculationError::Overflow {
                step: "withholding_tax_total".into(),
            }
        })?;
        breakdown.push(WithholdingTaxStep {
            label: format!("{}円超の部分", threshold),
            base_amount: excess_amount,
            rate_numer: excess_rate_numer,
            rate_denom: excess_rate_denom,
            result: excess_tax,
        });
    }

    Ok((FinalAmount::new(total_tax), breakdown))
}

fn apply_rate(
    amount: u64,
    rate_numer: u64,
    rate_denom: u64,
    policy: &dyn WithholdingTaxPolicy,
) -> Result<FinalAmount, JLawError> {
    let rate = Rate {
        numer: rate_numer,
        denom: rate_denom,
    };
    let rounding = policy.tax_rounding();
    Ok(rate
        .apply(
            &IntermediateAmount::from_exact(amount),
            MultiplyOrder::MultiplyFirst,
            rounding,
        )?
        .finalize(rounding)?)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::withholding_tax::params::WithholdingTaxCategoryParams;
    use crate::domains::withholding_tax::policy::StandardWithholdingTaxPolicy;

    #[test]
    fn zero_amount_returns_zero() {
        let params = WithholdingTaxParams {
            categories: vec![WithholdingTaxCategoryParams {
                category: WithholdingTaxCategory::ProfessionalFee,
                label: "税理士等の報酬".into(),
                method: WithholdingTaxMethod::TwoTier {
                    threshold: 1_000_000,
                    base_rate_numer: 1021,
                    base_rate_denom: 10_000,
                    excess_rate_numer: 2042,
                    excess_rate_denom: 10_000,
                },
                submission_prize_exemption_threshold: None,
            }],
        };
        let ctx = WithholdingTaxContext {
            payment_amount: 0,
            separated_consumption_tax_amount: 0,
            category: WithholdingTaxCategory::ProfessionalFee,
            target_date: crate::LegalDate::new(2026, 1, 1),
            flags: HashSet::new(),
            policy: Box::new(StandardWithholdingTaxPolicy),
        };

        let result = calculate_withholding_tax(&ctx, &params).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 0);
        assert_eq!(result.net_payment_amount.as_yen(), 0);
    }
}
