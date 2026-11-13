use crate::domains::income_tax::deduction::context::IncomeDeductionContext;
use crate::domains::income_tax::deduction::params::{
    BasicDeductionParams, DependentDeductionParams, PersonalDeductionParams,
    SpecificRelativeDeductionParams, SpouseDeductionParams,
};
use crate::domains::income_tax::deduction::types::{IncomeDeductionKind, IncomeDeductionLine};
use crate::error::{CalculationError, JLawError};
use crate::types::amount::FinalAmount;

pub(crate) fn calculate_personal_deductions(
    ctx: &IncomeDeductionContext,
    params: &PersonalDeductionParams,
) -> Result<Vec<IncomeDeductionLine>, JLawError> {
    let mut lines = vec![
        calculate_basic_deduction(ctx, &params.basic)?,
        calculate_spouse_deduction(ctx, &params.spouse)?,
        calculate_dependent_deduction(ctx, &params.dependent)?,
    ];
    // 特定親族特別控除は制度創設（2025年分）以後の年分でのみ内訳に含める。
    if let Some(specific_relative) = &params.specific_relative {
        lines.push(calculate_specific_relative_deduction(
            ctx,
            specific_relative,
        )?);
    }
    Ok(lines)
}

fn calculate_basic_deduction(
    ctx: &IncomeDeductionContext,
    params: &BasicDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let bracket = params
        .brackets
        .iter()
        .find(|bracket| {
            in_income_range(
                ctx.total_income_amount,
                bracket.income_from,
                bracket.income_to_inclusive,
            )
        })
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
        .find(|bracket| {
            in_income_range(
                ctx.total_income_amount,
                bracket.taxpayer_income_from,
                bracket.taxpayer_income_to_inclusive,
            )
        })
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

/// 特定親族特別控除額を計算する。
///
/// 特定親族ごとに合計所得金額から控除額を引き当てて合算する。
/// 控除額表の範囲外（扶養親族の所得要件以下、または123万円超）の特定親族は0円とする。
fn calculate_specific_relative_deduction(
    ctx: &IncomeDeductionContext,
    params: &SpecificRelativeDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let amount =
        ctx.deductions
            .personal
            .specific_relatives
            .iter()
            .try_fold(0_u64, |acc, relative| {
                let relative_amount = params
                    .brackets
                    .iter()
                    .find(|bracket| {
                        in_income_range(
                            relative.total_income_amount,
                            bracket.income_from,
                            bracket.income_to_inclusive,
                        )
                    })
                    .map_or(0, |bracket| bracket.deduction_amount);
                acc.checked_add(relative_amount)
                    .ok_or_else(|| CalculationError::Overflow {
                        step: "specific_relative_total".into(),
                    })
            })?;

    Ok(IncomeDeductionLine {
        kind: IncomeDeductionKind::SpecificRelative,
        label: "特定親族特別控除".into(),
        amount: FinalAmount::new(amount),
    })
}

fn in_income_range(income: u64, from: u64, to_inclusive: Option<u64>) -> bool {
    income >= from
        && match to_inclusive {
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
        PersonalDeductionInput, SpecificRelativeDeductionInput, SpouseDeductionInput,
    };
    use crate::domains::income_tax::deduction::params::{
        BasicDeductionBracket, DependentDeductionParams, SpecificRelativeDeductionBracket,
        SpecificRelativeDeductionParams, SpouseIncomeBracket,
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
            specific_relative: Some(specific_relative_params()),
        }
    }

    /// 令和8年分以後の特定親族特別控除額表（国税庁 タックスアンサー No.1177）。
    fn specific_relative_params() -> SpecificRelativeDeductionParams {
        let rows = [
            (620_001, 850_000, 630_000),
            (850_001, 900_000, 610_000),
            (900_001, 950_000, 510_000),
            (950_001, 1_000_000, 410_000),
            (1_000_001, 1_050_000, 310_000),
            (1_050_001, 1_100_000, 210_000),
            (1_100_001, 1_150_000, 110_000),
            (1_150_001, 1_200_000, 60_000),
            (1_200_001, 1_230_000, 30_000),
        ];
        SpecificRelativeDeductionParams {
            brackets: rows
                .iter()
                .map(|&(from, to, amount)| SpecificRelativeDeductionBracket {
                    label: format!("{from}円以上{to}円以下"),
                    income_from: from,
                    income_to_inclusive: Some(to),
                    deduction_amount: amount,
                })
                .collect(),
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
                personal: PersonalDeductionInput {
                    spouse,
                    dependent,
                    specific_relatives: vec![],
                },
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

    fn ctx_with_relatives(incomes: &[u64]) -> IncomeDeductionContext {
        let mut ctx = ctx(5_000_000, None, DependentDeductionInput::default());
        ctx.deductions.personal.specific_relatives = incomes
            .iter()
            .map(|&total_income_amount| SpecificRelativeDeductionInput {
                total_income_amount,
            })
            .collect();
        ctx
    }

    fn specific_relative_line(lines: &[IncomeDeductionLine]) -> &IncomeDeductionLine {
        lines
            .iter()
            .find(|line| line.kind == IncomeDeductionKind::SpecificRelative)
            .unwrap()
    }

    #[test]
    fn specific_relative_deduction_thresholds() {
        let cases = [
            (620_000, 0),
            (620_001, 630_000),
            (850_000, 630_000),
            (850_001, 610_000),
            (1_000_000, 410_000),
            (1_230_000, 30_000),
            (1_230_001, 0),
        ];

        for (income, expected) in cases {
            let result =
                calculate_personal_deductions(&ctx_with_relatives(&[income]), &params()).unwrap();
            assert_eq!(
                specific_relative_line(&result).amount.as_yen(),
                expected,
                "特定親族の合計所得金額 {income}円"
            );
        }
    }

    #[test]
    fn specific_relative_deduction_sums_each_relative() {
        let result =
            calculate_personal_deductions(&ctx_with_relatives(&[700_000, 1_100_000]), &params())
                .unwrap();
        assert_eq!(
            specific_relative_line(&result).amount.as_yen(),
            630_000 + 210_000
        );
    }

    #[test]
    fn specific_relative_deduction_is_zero_without_relatives() {
        let result = calculate_personal_deductions(&ctx_with_relatives(&[]), &params()).unwrap();
        assert_eq!(specific_relative_line(&result).amount.as_yen(), 0);
    }

    #[test]
    fn specific_relative_line_follows_dependent_line() {
        let result = calculate_personal_deductions(&ctx_with_relatives(&[]), &params()).unwrap();
        let kinds = result.iter().map(|line| line.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                IncomeDeductionKind::Basic,
                IncomeDeductionKind::Spouse,
                IncomeDeductionKind::Dependent,
                IncomeDeductionKind::SpecificRelative,
            ]
        );
    }

    #[test]
    fn specific_relative_line_is_omitted_before_enactment() {
        let mut params = params();
        params.specific_relative = None;
        let result =
            calculate_personal_deductions(&ctx_with_relatives(&[700_000]), &params).unwrap();
        assert!(result
            .iter()
            .all(|line| line.kind != IncomeDeductionKind::SpecificRelative));
        assert_eq!(result.len(), 3);
    }
}
