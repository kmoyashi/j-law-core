use crate::domains::income_tax::deduction::context::IncomeDeductionContext;
use crate::domains::income_tax::deduction::expense::calculate_expense_deductions;
use crate::domains::income_tax::deduction::params::IncomeDeductionParams;
use crate::domains::income_tax::deduction::personal::calculate_personal_deductions;
use crate::domains::income_tax::deduction::types::{IncomeDeductionLine, IncomeDeductionResult};
use crate::error::{CalculationError, JLawError};
use crate::types::amount::FinalAmount;

/// 総所得金額等から所得控除額を差し引き、課税所得金額を計算する。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
pub fn calculate_income_deductions(
    ctx: &IncomeDeductionContext,
    params: &IncomeDeductionParams,
) -> Result<IncomeDeductionResult, JLawError> {
    let mut breakdown = calculate_personal_deductions(ctx, &params.personal)?;
    breakdown.extend(calculate_expense_deductions(ctx, &params.expense)?);

    let total_deductions_yen = sum_deductions(&breakdown)?;
    let taxable_income_before_truncation =
        ctx.total_income_amount.saturating_sub(total_deductions_yen);
    let taxable_income = truncate_below_1000(taxable_income_before_truncation);

    Ok(IncomeDeductionResult {
        total_income_amount: FinalAmount::new(ctx.total_income_amount),
        total_deductions: FinalAmount::new(total_deductions_yen),
        taxable_income_before_truncation: FinalAmount::new(taxable_income_before_truncation),
        taxable_income: FinalAmount::new(taxable_income),
        breakdown,
    })
}

fn sum_deductions(lines: &[IncomeDeductionLine]) -> Result<u64, JLawError> {
    lines.iter().try_fold(0_u64, |acc, line| {
        acc.checked_add(line.amount.as_yen())
            .ok_or_else(|| CalculationError::Overflow {
                step: "income_deduction_total".into(),
            })
            .map_err(JLawError::from)
    })
}

fn truncate_below_1000(amount: u64) -> u64 {
    amount / 1_000 * 1_000
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::income_tax::deduction::context::{
        DependentDeductionInput, ExpenseDeductionInput, IncomeDeductionInput,
        PersonalDeductionInput,
    };
    use crate::domains::income_tax::deduction::params::{
        BasicDeductionBracket, BasicDeductionParams, DependentDeductionParams,
        DonationDeductionParams, ExpenseDeductionParams, LifeInsuranceDeductionBracket,
        LifeInsuranceDeductionParams, MedicalDeductionParams, PersonalDeductionParams,
        SocialInsuranceDeductionParams, SpouseDeductionParams,
    };
    use crate::types::date::LegalDate;

    fn params() -> IncomeDeductionParams {
        IncomeDeductionParams {
            personal: PersonalDeductionParams {
                basic: BasicDeductionParams {
                    brackets: vec![BasicDeductionBracket {
                        label: "2,400万円以下".into(),
                        income_from: 0,
                        income_to_inclusive: None,
                        deduction_amount: 480_000,
                    }],
                },
                spouse: SpouseDeductionParams {
                    qualifying_spouse_income_max: 480_000,
                    taxpayer_income_brackets: vec![],
                },
                dependent: DependentDeductionParams {
                    general_deduction_amount: 380_000,
                    specific_deduction_amount: 630_000,
                    elderly_cohabiting_deduction_amount: 580_000,
                    elderly_other_deduction_amount: 480_000,
                },
            },
            expense: ExpenseDeductionParams {
                social_insurance: SocialInsuranceDeductionParams,
                medical: MedicalDeductionParams {
                    income_threshold_rate_numer: 5,
                    income_threshold_rate_denom: 100,
                    threshold_cap_amount: 100_000,
                    deduction_cap_amount: 2_000_000,
                },
                life_insurance: LifeInsuranceDeductionParams {
                    new_contract_brackets: vec![LifeInsuranceDeductionBracket {
                        label: "8万円超".into(),
                        paid_from: 0,
                        paid_to_inclusive: None,
                        rate_numer: 0,
                        rate_denom: 1,
                        addition_amount: 40_000,
                        deduction_cap_amount: 40_000,
                    }],
                    old_contract_brackets: vec![LifeInsuranceDeductionBracket {
                        label: "10万円超".into(),
                        paid_from: 0,
                        paid_to_inclusive: None,
                        rate_numer: 0,
                        rate_denom: 1,
                        addition_amount: 50_000,
                        deduction_cap_amount: 50_000,
                    }],
                    mixed_contract_cap_amount: 40_000,
                    new_contract_cap_amount: 40_000,
                    old_contract_cap_amount: 50_000,
                    combined_cap_amount: 120_000,
                },
                donation: DonationDeductionParams {
                    income_cap_rate_numer: 40,
                    income_cap_rate_denom: 100,
                    non_deductible_amount: 2_000,
                },
            },
        }
    }

    fn ctx(total_income_amount: u64, social_insurance_premium_paid: u64) -> IncomeDeductionContext {
        IncomeDeductionContext {
            total_income_amount,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid,
                    medical: None,
                    life_insurance: None,
                    donation: None,
                },
            },
        }
    }

    #[test]
    fn truncates_taxable_income_below_1000() {
        let result = calculate_income_deductions(&ctx(481_999, 0), &params()).unwrap();
        assert_eq!(result.taxable_income_before_truncation.as_yen(), 1_999);
        assert_eq!(result.taxable_income.as_yen(), 1_000);
    }

    #[test]
    fn taxable_income_does_not_go_below_zero() {
        let result = calculate_income_deductions(&ctx(100_000, 30_000), &params()).unwrap();
        assert_eq!(result.taxable_income_before_truncation.as_yen(), 0);
        assert_eq!(result.taxable_income.as_yen(), 0);
    }
}
