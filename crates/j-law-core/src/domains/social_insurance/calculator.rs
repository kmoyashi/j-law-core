use std::collections::HashSet;

use crate::domains::social_insurance::context::SocialInsuranceContext;
use crate::domains::social_insurance::params::{SocialInsuranceParams, SocialInsuranceRate};
use crate::domains::social_insurance::policy::EmployeeShareRoundingMode;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};

use super::context::SocialInsuranceFlag;

/// 社会保険料の計算内訳。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
#[derive(Debug, Clone)]
pub struct SocialInsuranceBreakdownStep {
    /// 内訳名称。
    pub label: String,
    /// 計算に用いた標準報酬月額。
    pub standard_monthly_remuneration: FinalAmount,
    /// 適用率の分子。
    pub rate_numer: u64,
    /// 適用率の分母。
    pub rate_denom: u64,
    /// 端数処理前の本人負担額。
    pub raw_employee_share: IntermediateAmount,
    /// 端数処理後の本人負担額。
    pub amount: FinalAmount,
}

/// 月額社会保険料の計算結果。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
#[derive(Debug, Clone)]
pub struct SocialInsuranceResult {
    /// 健康保険料（介護該当時は介護保険料を含む）の本人負担額。
    pub health_related_amount: FinalAmount,
    /// 厚生年金保険料の本人負担額。
    pub pension_amount: FinalAmount,
    /// 本人負担合計額。
    pub total_amount: FinalAmount,
    /// 健康保険側の標準報酬月額。
    pub health_standard_monthly_remuneration: FinalAmount,
    /// 厚生年金側の標準報酬月額。
    pub pension_standard_monthly_remuneration: FinalAmount,
    /// 介護保険料を合算したか。
    pub care_insurance_applied: bool,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<SocialInsuranceFlag>,
    /// 計算内訳。
    pub breakdown: Vec<SocialInsuranceBreakdownStep>,
}

/// 協会けんぽ一般被保険者の月額社会保険料本人負担分を計算する。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
/// 日本年金機構 FAQ（被保険者負担分の50銭端数処理）
pub fn calculate_social_insurance_premium(
    ctx: &SocialInsuranceContext,
    params: &SocialInsuranceParams,
) -> Result<SocialInsuranceResult, JLawError> {
    if !params
        .valid_standard_monthly_remunerations
        .contains(&ctx.standard_monthly_remuneration)
    {
        return Err(CalculationError::PolicyNotApplicable {
            reason: format!(
                "標準報酬月額 {}円 は協会けんぽの標準報酬月額表に存在しません",
                ctx.standard_monthly_remuneration
            ),
        }
        .into());
    }

    let health_rate = params
        .prefecture_health_rates
        .iter()
        .find(|entry| entry.prefecture == ctx.prefecture)
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "都道府県コード {} に対応する健康保険料率が見つかりません",
                ctx.prefecture.code()
            ),
        })?
        .rate;

    let care_applied = ctx.policy.should_apply_care_insurance(&ctx.flags);
    let health_related_rate = if care_applied {
        SocialInsuranceRate {
            numer: health_rate
                .numer
                .checked_add(params.care_rate.numer)
                .ok_or_else(|| CalculationError::Overflow {
                    step: "social_insurance_health_related_rate".into(),
                })?,
            denom: health_rate.denom,
        }
    } else {
        health_rate
    };

    let rounding_mode = ctx.policy.employee_share_rounding_mode();
    let health_raw = calculate_raw_employee_share(
        ctx.standard_monthly_remuneration,
        health_related_rate,
        "social_insurance_health_related",
    )?;
    let health_amount = round_employee_share(
        &health_raw,
        rounding_mode,
        "social_insurance_health_related",
    )?;

    let pension_standard_monthly_remuneration = ctx
        .standard_monthly_remuneration
        .min(params.pension_standard_monthly_remuneration_cap);
    let pension_raw = calculate_raw_employee_share(
        pension_standard_monthly_remuneration,
        params.pension_rate,
        "social_insurance_pension",
    )?;
    let pension_amount =
        round_employee_share(&pension_raw, rounding_mode, "social_insurance_pension")?;

    let total_amount = FinalAmount::new(
        health_amount
            .as_yen()
            .checked_add(pension_amount.as_yen())
            .ok_or_else(|| CalculationError::Overflow {
                step: "social_insurance_total".into(),
            })?,
    );

    let health_label = if care_applied {
        "健康保険料・介護保険料"
    } else {
        "健康保険料"
    };

    Ok(SocialInsuranceResult {
        health_related_amount: health_amount,
        pension_amount,
        total_amount,
        health_standard_monthly_remuneration: FinalAmount::new(ctx.standard_monthly_remuneration),
        pension_standard_monthly_remuneration: FinalAmount::new(
            pension_standard_monthly_remuneration,
        ),
        care_insurance_applied: care_applied,
        applied_flags: ctx.flags.clone(),
        breakdown: vec![
            SocialInsuranceBreakdownStep {
                label: health_label.into(),
                standard_monthly_remuneration: FinalAmount::new(ctx.standard_monthly_remuneration),
                rate_numer: health_related_rate.numer,
                rate_denom: health_related_rate.denom,
                raw_employee_share: health_raw,
                amount: health_amount,
            },
            SocialInsuranceBreakdownStep {
                label: "厚生年金保険料".into(),
                standard_monthly_remuneration: FinalAmount::new(
                    pension_standard_monthly_remuneration,
                ),
                rate_numer: params.pension_rate.numer,
                rate_denom: params.pension_rate.denom,
                raw_employee_share: pension_raw,
                amount: pension_amount,
            },
        ],
    })
}

fn calculate_raw_employee_share(
    standard_monthly_remuneration: u64,
    rate: SocialInsuranceRate,
    step: &str,
) -> Result<IntermediateAmount, JLawError> {
    let numerator = standard_monthly_remuneration
        .checked_mul(rate.numer)
        .ok_or_else(|| CalculationError::Overflow { step: step.into() })?;
    let denominator = rate
        .denom
        .checked_mul(2)
        .ok_or_else(|| CalculationError::Overflow { step: step.into() })?;
    let whole = numerator / denominator;
    let numer = numerator % denominator;
    IntermediateAmount::try_new(whole, numer, denominator).map_err(JLawError::from)
}

fn round_employee_share(
    raw: &IntermediateAmount,
    mode: EmployeeShareRoundingMode,
    step: &str,
) -> Result<FinalAmount, JLawError> {
    if raw.numer == 0 {
        return Ok(FinalAmount::new(raw.whole));
    }

    let doubled_remainder = raw
        .numer
        .checked_mul(2)
        .ok_or_else(|| CalculationError::Overflow { step: step.into() })?;
    let should_round_up = if doubled_remainder < raw.denom {
        false
    } else if doubled_remainder > raw.denom {
        true
    } else {
        matches!(mode, EmployeeShareRoundingMode::CashPayment)
    };

    let amount = if should_round_up {
        raw.whole
            .checked_add(1)
            .ok_or_else(|| CalculationError::Overflow { step: step.into() })?
    } else {
        raw.whole
    };

    Ok(FinalAmount::new(amount))
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::collections::HashSet;

    use crate::domains::social_insurance::context::{
        SocialInsuranceContext, SocialInsuranceFlag, SocialInsurancePrefecture,
    };
    use crate::domains::social_insurance::params::{
        PrefectureHealthInsuranceRate, SocialInsuranceParams, SocialInsuranceRate,
    };
    use crate::domains::social_insurance::policy::{
        EmployeeShareRoundingMode, SocialInsurancePolicy, StandardNenkinPolicy,
    };
    use crate::LegalDate;

    use super::*;

    fn params() -> SocialInsuranceParams {
        SocialInsuranceParams {
            prefecture_health_rates: vec![PrefectureHealthInsuranceRate {
                prefecture: SocialInsurancePrefecture::Tokyo,
                rate: SocialInsuranceRate {
                    numer: 985,
                    denom: 10_000,
                },
            }],
            care_rate: SocialInsuranceRate {
                numer: 162,
                denom: 10_000,
            },
            pension_rate: SocialInsuranceRate {
                numer: 1_830,
                denom: 10_000,
            },
            valid_standard_monthly_remunerations: vec![118_000, 150_000, 680_000],
            pension_standard_monthly_remuneration_cap: 650_000,
        }
    }

    fn ctx(
        standard_monthly_remuneration: u64,
        flags: HashSet<SocialInsuranceFlag>,
        policy: Box<dyn SocialInsurancePolicy>,
    ) -> SocialInsuranceContext {
        SocialInsuranceContext {
            standard_monthly_remuneration,
            target_date: LegalDate::new(2026, 3, 1),
            prefecture: SocialInsurancePrefecture::Tokyo,
            flags,
            policy,
        }
    }

    #[test]
    fn payroll_deduction_floors_exact_half_yen() {
        let result = calculate_social_insurance_premium(
            &ctx(118_000, HashSet::new(), Box::new(StandardNenkinPolicy)),
            &params(),
        )
        .unwrap();
        assert_eq!(result.health_related_amount.as_yen(), 5_811);
        assert_eq!(result.pension_amount.as_yen(), 10_797);
        assert_eq!(result.total_amount.as_yen(), 16_608);
    }

    #[derive(Debug)]
    struct CashPolicy;

    impl SocialInsurancePolicy for CashPolicy {
        fn should_apply_care_insurance(&self, flags: &HashSet<SocialInsuranceFlag>) -> bool {
            flags.contains(&SocialInsuranceFlag::IsCareInsuranceApplicable)
        }

        fn employee_share_rounding_mode(&self) -> EmployeeShareRoundingMode {
            EmployeeShareRoundingMode::CashPayment
        }
    }

    #[test]
    fn cash_payment_ceils_exact_half_yen() {
        let result = calculate_social_insurance_premium(
            &ctx(118_000, HashSet::new(), Box::new(CashPolicy)),
            &params(),
        )
        .unwrap();
        assert_eq!(result.health_related_amount.as_yen(), 5_812);
    }

    #[test]
    fn care_rate_is_added_before_rounding() {
        let mut flags = HashSet::new();
        flags.insert(SocialInsuranceFlag::IsCareInsuranceApplicable);
        let result = calculate_social_insurance_premium(
            &ctx(150_000, flags, Box::new(StandardNenkinPolicy)),
            &params(),
        )
        .unwrap();
        assert_eq!(result.health_related_amount.as_yen(), 8_602);
        assert_eq!(result.total_amount.as_yen(), 22_327);
        assert_eq!(result.breakdown[0].rate_numer, 1_147);
    }

    #[test]
    fn pension_uses_cap() {
        let result = calculate_social_insurance_premium(
            &ctx(680_000, HashSet::new(), Box::new(StandardNenkinPolicy)),
            &params(),
        )
        .unwrap();
        assert_eq!(
            result.pension_standard_monthly_remuneration.as_yen(),
            650_000
        );
        assert_eq!(result.pension_amount.as_yen(), 59_475);
    }
}
