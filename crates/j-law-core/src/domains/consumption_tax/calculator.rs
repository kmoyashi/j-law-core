use std::collections::HashSet;

use crate::domains::consumption_tax::context::{ConsumptionTaxContext, ConsumptionTaxFlag};
use crate::domains::consumption_tax::params::ConsumptionTaxParams;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};
use crate::types::rate::{MultiplyOrder, Rate};

/// 消費税の計算結果。
#[derive(Debug, Clone)]
pub struct ConsumptionTaxResult {
    /// 消費税額（切り捨て後）。
    pub tax_amount: FinalAmount,
    /// 税込み金額（課税標準額 + 消費税額）。
    pub amount_with_tax: FinalAmount,
    /// 課税標準額（税抜き）。`ctx.amount` と等しい。
    pub amount_without_tax: FinalAmount,
    /// 適用した税率の分子。
    pub applied_rate_numer: u64,
    /// 適用した税率の分母。
    pub applied_rate_denom: u64,
    /// 軽減税率が適用されたか。
    pub is_reduced_rate: bool,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<ConsumptionTaxFlag>,
}

/// 消費税法第29条に基づく消費税を計算する。
///
/// # 法的根拠
/// 消費税法 第29条（税率）
/// 消費税法 第45条（端数処理: 1円未満切り捨て）
///
/// # 計算手順
/// 1. ポリシーで軽減税率を適用するか判定する
/// 2. 軽減税率適用フラグがあっても `reduced_rate` が `None` の場合は
///    [`CalculationError::PolicyNotApplicable`] を返す
/// 3. 課税標準額 × 税率（切り捨て）で消費税額を算出する
/// 4. 課税標準額 + 消費税額で税込み金額を算出する
///
/// # 税率が0の場合（消費税導入前）
/// `standard_rate.numer == 0` のとき、税額は0として計算結果を返す。
pub fn calculate_consumption_tax(
    ctx: &ConsumptionTaxContext,
    params: &ConsumptionTaxParams,
) -> Result<ConsumptionTaxResult, JLawError> {
    let amount = ctx.amount;
    let rounding = ctx.policy.tax_rounding();
    let use_reduced = ctx.policy.should_apply_reduced_rate(&ctx.flags);

    let (applied_rate, is_reduced) = if use_reduced {
        let reduced = params.reduced_rate.ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "軽減税率フラグが指定されましたが、対象日({})の消費税パラメータに軽減税率が存在しません",
                ctx.target_date.to_date_str()
            ),
        })?;
        (reduced, true)
    } else {
        (params.standard_rate, false)
    };

    // 税率が0の場合（消費税導入前）は税額0を返す
    let tax_amount = if applied_rate.numer == 0 {
        FinalAmount::new(0)
    } else {
        let rate = Rate {
            numer: applied_rate.numer,
            denom: applied_rate.denom,
        };
        rate.apply(
            &IntermediateAmount::from_exact(amount),
            MultiplyOrder::MultiplyFirst,
            rounding,
        )?
        .finalize(rounding)?
    };

    let amount_with_tax =
        FinalAmount::new(amount.checked_add(tax_amount.as_yen()).ok_or_else(|| {
            CalculationError::Overflow {
                step: "consumption_tax".into(),
            }
        })?);

    Ok(ConsumptionTaxResult {
        tax_amount,
        amount_with_tax,
        amount_without_tax: FinalAmount::new(amount),
        applied_rate_numer: applied_rate.numer,
        applied_rate_denom: applied_rate.denom,
        is_reduced_rate: is_reduced,
        applied_flags: ctx.flags.clone(),
    })
}


#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::consumption_tax::params::ConsumptionTaxRate;
    use crate::domains::consumption_tax::policy::StandardConsumptionTaxPolicy;
    use crate::LegalDate;

    fn params_10pct() -> ConsumptionTaxParams {
        ConsumptionTaxParams {
            standard_rate: ConsumptionTaxRate {
                numer: 10,
                denom: 100,
            },
            reduced_rate: Some(ConsumptionTaxRate {
                numer: 8,
                denom: 100,
            }),
        }
    }

    fn ctx_standard(amount: u64) -> ConsumptionTaxContext {
        ConsumptionTaxContext {
            amount,
            target_date: LegalDate::new(2020, 1, 1),
            flags: HashSet::new(),
            policy: Box::new(StandardConsumptionTaxPolicy),
        }
    }

    fn ctx_reduced(amount: u64) -> ConsumptionTaxContext {
        let mut flags = HashSet::new();
        flags.insert(ConsumptionTaxFlag::ReducedRate);
        ConsumptionTaxContext {
            amount,
            target_date: LegalDate::new(2020, 1, 1),
            flags,
            policy: Box::new(StandardConsumptionTaxPolicy),
        }
    }

    #[test]
    fn standard_10pct() {
        let result = calculate_consumption_tax(&ctx_standard(100_000), &params_10pct()).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 10_000);
        assert_eq!(result.amount_with_tax.as_yen(), 110_000);
        assert_eq!(result.amount_without_tax.as_yen(), 100_000);
        assert!(!result.is_reduced_rate);
    }

    #[test]
    fn reduced_8pct() {
        let result = calculate_consumption_tax(&ctx_reduced(100_000), &params_10pct()).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 8_000);
        assert_eq!(result.amount_with_tax.as_yen(), 108_000);
        assert!(result.is_reduced_rate);
    }

    #[test]
    fn floor_rounding() {
        // 100,001 × 10% = 10,000.1 → 切り捨て → 10,000
        let result = calculate_consumption_tax(&ctx_standard(100_001), &params_10pct()).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 10_000);
    }

    #[test]
    fn zero_amount() {
        let result = calculate_consumption_tax(&ctx_standard(0), &params_10pct()).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 0);
        assert_eq!(result.amount_with_tax.as_yen(), 0);
    }

    #[test]
    fn zero_rate_no_tax() {
        let params = ConsumptionTaxParams {
            standard_rate: ConsumptionTaxRate { numer: 0, denom: 100 },
            reduced_rate: None,
        };
        let result = calculate_consumption_tax(&ctx_standard(100_000), &params).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 0);
        assert_eq!(result.amount_with_tax.as_yen(), 100_000);
    }

    #[test]
    fn reduced_flag_without_reduced_rate_is_error() {
        let params = ConsumptionTaxParams {
            standard_rate: ConsumptionTaxRate {
                numer: 8,
                denom: 100,
            },
            reduced_rate: None, // 軽減税率なし
        };
        let result = calculate_consumption_tax(&ctx_reduced(100_000), &params);
        assert!(matches!(
            result,
            Err(JLawError::Calculation(
                crate::error::CalculationError::PolicyNotApplicable { .. }
            ))
        ));
    }
}
