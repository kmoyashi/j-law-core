use crate::domains::income_tax::deduction::context::{
    ExpenseDeductionInput, IncomeDeductionContext,
};
use crate::domains::income_tax::deduction::params::{
    DonationDeductionParams, ExpenseDeductionParams, LifeInsuranceDeductionBracket,
    LifeInsuranceDeductionParams, MedicalDeductionParams,
};
use crate::domains::income_tax::deduction::types::{IncomeDeductionKind, IncomeDeductionLine};
use crate::error::{CalculationError, InputError, JLawError};
use crate::types::amount::FinalAmount;

pub(crate) fn calculate_expense_deductions(
    ctx: &IncomeDeductionContext,
    params: &ExpenseDeductionParams,
) -> Result<Vec<IncomeDeductionLine>, JLawError> {
    Ok(vec![
        calculate_social_insurance_deduction(&ctx.deductions.expense),
        calculate_medical_deduction(ctx, &params.medical)?,
        calculate_life_insurance_deduction(ctx, &params.life_insurance)?,
        calculate_donation_deduction(ctx, &params.donation)?,
    ])
}

fn calculate_social_insurance_deduction(expense: &ExpenseDeductionInput) -> IncomeDeductionLine {
    IncomeDeductionLine {
        kind: IncomeDeductionKind::SocialInsurance,
        label: "社会保険料控除".into(),
        amount: FinalAmount::new(expense.social_insurance_premium_paid),
    }
}

fn calculate_medical_deduction(
    ctx: &IncomeDeductionContext,
    params: &MedicalDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let Some(medical) = ctx.deductions.expense.medical else {
        return Ok(zero_line(IncomeDeductionKind::Medical, "医療費控除"));
    };

    let net_paid = medical
        .medical_expense_paid
        .checked_sub(medical.reimbursed_amount)
        .ok_or(InputError::InvalidDeductionInput {
            field: "medical.reimbursed_amount".into(),
            reason: "補填額が支払医療費を上回っています".into(),
        })?;

    let income_threshold = calculate_rate_amount(
        ctx.total_income_amount,
        params.income_threshold_rate_numer,
        params.income_threshold_rate_denom,
        "medical_income_threshold",
    )?;
    let threshold = income_threshold.min(params.threshold_cap_amount);
    let deduction_amount = net_paid
        .saturating_sub(threshold)
        .min(params.deduction_cap_amount);

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::Medical,
        label: "医療費控除".into(),
        amount: FinalAmount::new(deduction_amount),
    })
}

fn calculate_life_insurance_deduction(
    ctx: &IncomeDeductionContext,
    params: &LifeInsuranceDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let Some(life_insurance) = ctx.deductions.expense.life_insurance else {
        return Ok(zero_line(
            IncomeDeductionKind::LifeInsurance,
            "生命保険料控除",
        ));
    };

    let general = calculate_life_insurance_component(
        life_insurance.new_general_paid_amount,
        life_insurance.old_general_paid_amount,
        params,
        "life_insurance_general",
    )?;
    let pension = calculate_life_insurance_component(
        life_insurance.new_individual_pension_paid_amount,
        life_insurance.old_individual_pension_paid_amount,
        params,
        "life_insurance_pension",
    )?;
    let care =
        calculate_life_insurance_new_amount(life_insurance.new_care_medical_paid_amount, params)?;

    let total = general
        .checked_add(pension)
        .ok_or_else(|| CalculationError::Overflow {
            step: "life_insurance_total".into(),
        })?
        .checked_add(care)
        .ok_or_else(|| CalculationError::Overflow {
            step: "life_insurance_total".into(),
        })?
        .min(params.combined_cap_amount);

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::LifeInsurance,
        label: "生命保険料控除".into(),
        amount: FinalAmount::new(total),
    })
}

fn calculate_donation_deduction(
    ctx: &IncomeDeductionContext,
    params: &DonationDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let Some(donation) = ctx.deductions.expense.donation else {
        return Ok(zero_line(IncomeDeductionKind::Donation, "寄附金控除"));
    };

    let income_cap = calculate_rate_amount(
        ctx.total_income_amount,
        params.income_cap_rate_numer,
        params.income_cap_rate_denom,
        "donation_income_cap",
    )?;
    let eligible_amount = donation.qualified_donation_amount.min(income_cap);
    let deduction_amount = eligible_amount.saturating_sub(params.non_deductible_amount);

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::Donation,
        label: "寄附金控除".into(),
        amount: FinalAmount::new(deduction_amount),
    })
}

fn calculate_life_insurance_component(
    new_paid_amount: u64,
    old_paid_amount: u64,
    params: &LifeInsuranceDeductionParams,
    overflow_step: &'static str,
) -> Result<u64, JLawError> {
    let new_amount =
        calculate_life_insurance_amount(new_paid_amount, &params.new_contract_brackets, "新契約")?;
    let old_amount =
        calculate_life_insurance_amount(old_paid_amount, &params.old_contract_brackets, "旧契約")?;

    let component_amount = if new_paid_amount > 0 && old_paid_amount > 0 {
        new_amount
            .checked_add(old_amount)
            .ok_or_else(|| CalculationError::Overflow {
                step: overflow_step.into(),
            })?
            .min(params.mixed_contract_cap_amount)
    } else if new_paid_amount > 0 {
        new_amount.min(params.new_contract_cap_amount)
    } else {
        old_amount.min(params.old_contract_cap_amount)
    };

    Ok(component_amount)
}

fn calculate_life_insurance_new_amount(
    paid_amount: u64,
    params: &LifeInsuranceDeductionParams,
) -> Result<u64, JLawError> {
    Ok(
        calculate_life_insurance_amount(paid_amount, &params.new_contract_brackets, "新契約")?
            .min(params.new_contract_cap_amount),
    )
}

fn calculate_life_insurance_amount(
    paid_amount: u64,
    brackets: &[LifeInsuranceDeductionBracket],
    contract_label: &'static str,
) -> Result<u64, JLawError> {
    if paid_amount == 0 {
        return Ok(0);
    }

    let bracket = brackets
        .iter()
        .find(|bracket| matches_life_insurance_bracket(paid_amount, bracket))
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "{}の生命保険料控除ブラケットが見つかりません: {}円",
                contract_label, paid_amount
            ),
        })?;

    let computed = calculate_rate_amount(
        paid_amount,
        bracket.rate_numer,
        bracket.rate_denom,
        "life_insurance_bracket",
    )?
    .checked_add(bracket.addition_amount)
    .ok_or_else(|| CalculationError::Overflow {
        step: "life_insurance_bracket".into(),
    })?;

    Ok(computed.min(bracket.deduction_cap_amount))
}

fn matches_life_insurance_bracket(
    paid_amount: u64,
    bracket: &LifeInsuranceDeductionBracket,
) -> bool {
    paid_amount >= bracket.paid_from
        && match bracket.paid_to_inclusive {
            Some(to) => paid_amount <= to,
            None => true,
        }
}

fn calculate_rate_amount(
    amount: u64,
    numer: u64,
    denom: u64,
    overflow_step: &'static str,
) -> Result<u64, JLawError> {
    if denom == 0 {
        return Err(InputError::ZeroDenominator.into());
    }

    let multiplied = amount
        .checked_mul(numer)
        .ok_or_else(|| CalculationError::Overflow {
            step: overflow_step.into(),
        })?;

    // SAFETY: denom != 0 はこの関数の先頭ガードで保証済み。
    Ok(multiplied / denom)
}

fn zero_line(kind: IncomeDeductionKind, label: &'static str) -> IncomeDeductionLine {
    IncomeDeductionLine {
        kind,
        label: label.into(),
        amount: FinalAmount::new(0),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::income_tax::deduction::context::{
        DependentDeductionInput, DonationDeductionInput, ExpenseDeductionInput,
        IncomeDeductionInput, LifeInsuranceDeductionInput, MedicalDeductionInput,
        PersonalDeductionInput,
    };
    use crate::domains::income_tax::deduction::params::{
        DonationDeductionParams, LifeInsuranceDeductionBracket, MedicalDeductionParams,
        SocialInsuranceDeductionParams,
    };
    use crate::types::date::LegalDate;

    fn params() -> ExpenseDeductionParams {
        ExpenseDeductionParams {
            social_insurance: SocialInsuranceDeductionParams,
            medical: MedicalDeductionParams {
                income_threshold_rate_numer: 5,
                income_threshold_rate_denom: 100,
                threshold_cap_amount: 100_000,
                deduction_cap_amount: 2_000_000,
            },
            life_insurance: LifeInsuranceDeductionParams {
                new_contract_brackets: vec![
                    LifeInsuranceDeductionBracket {
                        label: "2万円以下".into(),
                        paid_from: 0,
                        paid_to_inclusive: Some(20_000),
                        rate_numer: 1,
                        rate_denom: 1,
                        addition_amount: 0,
                        deduction_cap_amount: 20_000,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "2万円超4万円以下".into(),
                        paid_from: 20_001,
                        paid_to_inclusive: Some(40_000),
                        rate_numer: 1,
                        rate_denom: 2,
                        addition_amount: 10_000,
                        deduction_cap_amount: 30_000,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "4万円超8万円以下".into(),
                        paid_from: 40_001,
                        paid_to_inclusive: Some(80_000),
                        rate_numer: 1,
                        rate_denom: 4,
                        addition_amount: 20_000,
                        deduction_cap_amount: 40_000,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "8万円超".into(),
                        paid_from: 80_001,
                        paid_to_inclusive: None,
                        rate_numer: 0,
                        rate_denom: 1,
                        addition_amount: 40_000,
                        deduction_cap_amount: 40_000,
                    },
                ],
                old_contract_brackets: vec![
                    LifeInsuranceDeductionBracket {
                        label: "2万5千円以下".into(),
                        paid_from: 0,
                        paid_to_inclusive: Some(25_000),
                        rate_numer: 1,
                        rate_denom: 1,
                        addition_amount: 0,
                        deduction_cap_amount: 25_000,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "2万5千円超5万円以下".into(),
                        paid_from: 25_001,
                        paid_to_inclusive: Some(50_000),
                        rate_numer: 1,
                        rate_denom: 2,
                        addition_amount: 12_500,
                        deduction_cap_amount: 37_500,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "5万円超10万円以下".into(),
                        paid_from: 50_001,
                        paid_to_inclusive: Some(100_000),
                        rate_numer: 1,
                        rate_denom: 4,
                        addition_amount: 25_000,
                        deduction_cap_amount: 50_000,
                    },
                    LifeInsuranceDeductionBracket {
                        label: "10万円超".into(),
                        paid_from: 100_001,
                        paid_to_inclusive: None,
                        rate_numer: 0,
                        rate_denom: 1,
                        addition_amount: 50_000,
                        deduction_cap_amount: 50_000,
                    },
                ],
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
        }
    }

    fn ctx(expense: ExpenseDeductionInput) -> IncomeDeductionContext {
        IncomeDeductionContext {
            total_income_amount: 5_000_000,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                },
                expense,
            },
        }
    }

    #[test]
    fn social_insurance_deduction_uses_full_paid_amount() {
        let result = calculate_expense_deductions(
            &ctx(ExpenseDeductionInput {
                social_insurance_premium_paid: 480_900,
                medical: None,
                life_insurance: None,
                donation: None,
            }),
            &params(),
        )
        .unwrap();
        assert_eq!(result[0].amount.as_yen(), 480_900);
    }

    #[test]
    fn medical_deduction_uses_five_percent_threshold_for_low_income() {
        let result = calculate_expense_deductions(
            &ctx(ExpenseDeductionInput {
                social_insurance_premium_paid: 0,
                medical: Some(MedicalDeductionInput {
                    medical_expense_paid: 400_000,
                    reimbursed_amount: 50_000,
                }),
                life_insurance: None,
                donation: None,
            }),
            &params(),
        )
        .unwrap();

        assert_eq!(result[1].amount.as_yen(), 250_000);
    }

    #[test]
    fn medical_deduction_rejects_reimbursements_above_paid_amount() {
        let result = calculate_expense_deductions(
            &ctx(ExpenseDeductionInput {
                social_insurance_premium_paid: 0,
                medical: Some(MedicalDeductionInput {
                    medical_expense_paid: 100_000,
                    reimbursed_amount: 100_001,
                }),
                life_insurance: None,
                donation: None,
            }),
            &params(),
        );

        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::InvalidDeductionInput { .. }))
        ));
    }

    #[test]
    fn life_insurance_deduction_caps_total_at_120k() {
        let result = calculate_expense_deductions(
            &ctx(ExpenseDeductionInput {
                social_insurance_premium_paid: 0,
                medical: None,
                life_insurance: Some(LifeInsuranceDeductionInput {
                    new_general_paid_amount: 100_000,
                    new_individual_pension_paid_amount: 100_000,
                    new_care_medical_paid_amount: 100_000,
                    old_general_paid_amount: 0,
                    old_individual_pension_paid_amount: 0,
                }),
                donation: None,
            }),
            &params(),
        )
        .unwrap();

        assert_eq!(result[2].amount.as_yen(), 120_000);
    }

    #[test]
    fn donation_deduction_uses_income_cap_and_self_burden() {
        let result = calculate_expense_deductions(
            &ctx(ExpenseDeductionInput {
                social_insurance_premium_paid: 0,
                medical: None,
                life_insurance: None,
                donation: Some(DonationDeductionInput {
                    qualified_donation_amount: 3_000_000,
                }),
            }),
            &params(),
        )
        .unwrap();

        assert_eq!(result[3].amount.as_yen(), 1_998_000);
    }
}
