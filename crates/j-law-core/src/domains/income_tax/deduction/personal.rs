use crate::domains::income_tax::deduction::context::IncomeDeductionContext;
use crate::domains::income_tax::deduction::params::{
    BasicDeductionBracket, BasicDeductionParams, DependentDeductionParams, PersonalDeductionParams,
    SpouseDeductionParams, SpouseIncomeBracket,
};
use crate::domains::income_tax::deduction::types::{IncomeDeductionKind, IncomeDeductionLine};
use crate::error::{CalculationError, JLawError};
use crate::types::amount::FinalAmount;

pub(crate) fn calculate_personal_deductions(
    ctx: &IncomeDeductionContext,
    params: &PersonalDeductionParams,
) -> Result<Vec<IncomeDeductionLine>, JLawError> {
    Ok(vec![
        calculate_basic_deduction(ctx, &params.basic)?,
        calculate_spouse_deduction(ctx, &params.spouse)?,
        calculate_dependent_deduction(ctx, &params.dependent)?,
    ])
}

fn calculate_basic_deduction(
    ctx: &IncomeDeductionContext,
    params: &BasicDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let bracket = params
        .brackets
        .iter()
        .find(|bracket| matches_basic_bracket(ctx.total_income_amount, bracket))
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "総所得金額等 {}円 に対応する基礎控除ブラケットが見つかりません",
                ctx.total_income_amount
            ),
        })?;

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::Basic,
        label: "基礎控除".into(),
        amount: FinalAmount::new(bracket.deduction_amount),
    })
}

fn calculate_spouse_deduction(
    ctx: &IncomeDeductionContext,
    params: &SpouseDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let Some(spouse) = ctx.deductions.personal.spouse else {
        return Ok(zero_line(IncomeDeductionKind::Spouse, "配偶者控除"));
    };

    if !spouse.is_same_household
        || spouse.spouse_total_income_amount > params.qualifying_spouse_income_max
    {
        return Ok(zero_line(IncomeDeductionKind::Spouse, "配偶者控除"));
    }

    let bracket = params
        .taxpayer_income_brackets
        .iter()
        .find(|bracket| matches_spouse_bracket(ctx.total_income_amount, bracket))
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "総所得金額等 {}円 に対応する配偶者控除ブラケットが見つかりません",
                ctx.total_income_amount
            ),
        })?;

    let amount = if spouse.is_elderly {
        bracket.elderly_deduction_amount
    } else {
        bracket.deduction_amount
    };

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::Spouse,
        label: "配偶者控除".into(),
        amount: FinalAmount::new(amount),
    })
}

fn calculate_dependent_deduction(
    ctx: &IncomeDeductionContext,
    params: &DependentDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let dependent = ctx.deductions.personal.dependent;
    let amount = multiply_count(
        dependent.general_count,
        params.general_deduction_amount,
        "dependent_general",
    )?
    .checked_add(multiply_count(
        dependent.specific_count,
        params.specific_deduction_amount,
        "dependent_specific",
    )?)
    .ok_or_else(|| CalculationError::Overflow {
        step: "dependent_total".into(),
    })?
    .checked_add(multiply_count(
        dependent.elderly_cohabiting_count,
        params.elderly_cohabiting_deduction_amount,
        "dependent_elderly_cohabiting",
    )?)
    .ok_or_else(|| CalculationError::Overflow {
        step: "dependent_total".into(),
    })?
    .checked_add(multiply_count(
        dependent.elderly_other_count,
        params.elderly_other_deduction_amount,
        "dependent_elderly_other",
    )?)
    .ok_or_else(|| CalculationError::Overflow {
        step: "dependent_total".into(),
    })?;

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::Dependent,
        label: "扶養控除".into(),
        amount: FinalAmount::new(amount),
    })
}

fn matches_basic_bracket(income: u64, bracket: &BasicDeductionBracket) -> bool {
    income >= bracket.income_from
        && match bracket.income_to_inclusive {
            Some(to) => income <= to,
            None => true,
        }
}

fn matches_spouse_bracket(income: u64, bracket: &SpouseIncomeBracket) -> bool {
    income >= bracket.taxpayer_income_from
        && match bracket.taxpayer_income_to_inclusive {
            Some(to) => income <= to,
            None => true,
        }
}

fn multiply_count(count: u16, amount: u64, step: &'static str) -> Result<u64, JLawError> {
    u64::from(count)
        .checked_mul(amount)
        .ok_or_else(|| CalculationError::Overflow { step: step.into() })
        .map_err(JLawError::from)
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
        DependentDeductionInput, ExpenseDeductionInput, IncomeDeductionInput,
        PersonalDeductionInput, SpouseDeductionInput,
    };
    use crate::domains::income_tax::deduction::params::{
        BasicDeductionBracket, DependentDeductionParams, SpouseIncomeBracket,
    };
    use crate::types::date::LegalDate;

    fn params() -> PersonalDeductionParams {
        PersonalDeductionParams {
            basic: BasicDeductionParams {
                brackets: vec![
                    BasicDeductionBracket {
                        label: "2,400万円以下".into(),
                        income_from: 0,
                        income_to_inclusive: Some(24_000_000),
                        deduction_amount: 480_000,
                    },
                    BasicDeductionBracket {
                        label: "2,400万円超2,450万円以下".into(),
                        income_from: 24_000_001,
                        income_to_inclusive: Some(24_500_000),
                        deduction_amount: 320_000,
                    },
                    BasicDeductionBracket {
                        label: "2,450万円超2,500万円以下".into(),
                        income_from: 24_500_001,
                        income_to_inclusive: Some(25_000_000),
                        deduction_amount: 160_000,
                    },
                    BasicDeductionBracket {
                        label: "2,500万円超".into(),
                        income_from: 25_000_001,
                        income_to_inclusive: None,
                        deduction_amount: 0,
                    },
                ],
            },
            spouse: SpouseDeductionParams {
                qualifying_spouse_income_max: 480_000,
                taxpayer_income_brackets: vec![
                    SpouseIncomeBracket {
                        label: "900万円以下".into(),
                        taxpayer_income_from: 0,
                        taxpayer_income_to_inclusive: Some(9_000_000),
                        deduction_amount: 380_000,
                        elderly_deduction_amount: 480_000,
                    },
                    SpouseIncomeBracket {
                        label: "900万円超950万円以下".into(),
                        taxpayer_income_from: 9_000_001,
                        taxpayer_income_to_inclusive: Some(9_500_000),
                        deduction_amount: 260_000,
                        elderly_deduction_amount: 320_000,
                    },
                    SpouseIncomeBracket {
                        label: "950万円超1000万円以下".into(),
                        taxpayer_income_from: 9_500_001,
                        taxpayer_income_to_inclusive: Some(10_000_000),
                        deduction_amount: 130_000,
                        elderly_deduction_amount: 160_000,
                    },
                    SpouseIncomeBracket {
                        label: "1000万円超".into(),
                        taxpayer_income_from: 10_000_001,
                        taxpayer_income_to_inclusive: None,
                        deduction_amount: 0,
                        elderly_deduction_amount: 0,
                    },
                ],
            },
            dependent: DependentDeductionParams {
                general_deduction_amount: 380_000,
                specific_deduction_amount: 630_000,
                elderly_cohabiting_deduction_amount: 580_000,
                elderly_other_deduction_amount: 480_000,
            },
        }
    }

    fn ctx(
        total_income_amount: u64,
        spouse: Option<SpouseDeductionInput>,
        dependent: DependentDeductionInput,
    ) -> IncomeDeductionContext {
        IncomeDeductionContext {
            total_income_amount,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput { spouse, dependent },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 0,
                    medical: None,
                    life_insurance: None,
                    donation: None,
                },
            },
        }
    }

    #[test]
    fn basic_deduction_thresholds() {
        let cases = [
            (24_000_000, 480_000),
            (24_000_001, 320_000),
            (24_500_001, 160_000),
            (25_000_001, 0),
        ];

        for (income, expected) in cases {
            let result = calculate_personal_deductions(
                &ctx(income, None, DependentDeductionInput::default()),
                &params(),
            )
            .unwrap();
            assert_eq!(result[0].amount.as_yen(), expected);
        }
    }

    #[test]
    fn spouse_deduction_thresholds() {
        let cases = [
            (9_000_000, 380_000),
            (9_000_001, 260_000),
            (9_500_001, 130_000),
            (10_000_001, 0),
        ];

        for (income, expected) in cases {
            let result = calculate_personal_deductions(
                &ctx(
                    income,
                    Some(SpouseDeductionInput {
                        spouse_total_income_amount: 480_000,
                        is_same_household: true,
                        is_elderly: false,
                    }),
                    DependentDeductionInput::default(),
                ),
                &params(),
            )
            .unwrap();
            assert_eq!(result[1].amount.as_yen(), expected);
        }
    }

    #[test]
    fn dependent_deduction_aggregates_all_categories() {
        let result = calculate_personal_deductions(
            &ctx(
                8_000_000,
                None,
                DependentDeductionInput {
                    general_count: 1,
                    specific_count: 1,
                    elderly_cohabiting_count: 1,
                    elderly_other_count: 1,
                },
            ),
            &params(),
        )
        .unwrap();

        assert_eq!(result[2].amount.as_yen(), 2_070_000);
    }
}
