use crate::domains::income_tax::deduction::context::IncomeDeductionContext;
use crate::domains::income_tax::deduction::params::{
    BasicDeductionParams, PersonalDeductionParams,
};
use crate::domains::income_tax::deduction::types::{IncomeDeductionKind, IncomeDeductionLine};
use crate::error::{CalculationError, JLawError};
use crate::types::amount::FinalAmount;

pub(crate) fn calculate_personal_deductions(
    ctx: &IncomeDeductionContext,
    params: &PersonalDeductionParams,
) -> Result<Vec<IncomeDeductionLine>, JLawError> {
    Ok(vec![calculate_basic_deduction(ctx, &params.basic)?])
}

fn calculate_basic_deduction(
    ctx: &IncomeDeductionContext,
    params: &BasicDeductionParams,
) -> Result<IncomeDeductionLine, JLawError> {
    let bracket = params
        .brackets
        .iter()
        .find(|bracket| {
            ctx.total_income_amount >= bracket.income_from
                && match bracket.income_to_inclusive {
                    Some(to) => ctx.total_income_amount <= to,
                    None => true,
                }
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

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::income_tax::deduction::context::{
        ExpenseDeductionInput, IncomeDeductionInput, PersonalDeductionInput,
    };
    use crate::types::date::LegalDate;

    fn params() -> PersonalDeductionParams {
        PersonalDeductionParams {
            basic: BasicDeductionParams {
                brackets: vec![
                    crate::domains::income_tax::deduction::params::BasicDeductionBracket {
                        label: "2,400万円以下".into(),
                        income_from: 0,
                        income_to_inclusive: Some(24_000_000),
                        deduction_amount: 480_000,
                    },
                    crate::domains::income_tax::deduction::params::BasicDeductionBracket {
                        label: "2,400万円超2,450万円以下".into(),
                        income_from: 24_000_001,
                        income_to_inclusive: Some(24_500_000),
                        deduction_amount: 320_000,
                    },
                    crate::domains::income_tax::deduction::params::BasicDeductionBracket {
                        label: "2,450万円超2,500万円以下".into(),
                        income_from: 24_500_001,
                        income_to_inclusive: Some(25_000_000),
                        deduction_amount: 160_000,
                    },
                    crate::domains::income_tax::deduction::params::BasicDeductionBracket {
                        label: "2,500万円超".into(),
                        income_from: 25_000_001,
                        income_to_inclusive: None,
                        deduction_amount: 0,
                    },
                ],
            },
        }
    }

    fn ctx(total_income_amount: u64) -> IncomeDeductionContext {
        IncomeDeductionContext {
            total_income_amount,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {},
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 0,
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
            let result = calculate_personal_deductions(&ctx(income), &params()).unwrap();
            assert_eq!(result[0].amount.as_yen(), expected);
        }
    }
}
