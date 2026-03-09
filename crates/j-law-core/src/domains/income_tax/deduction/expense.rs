use crate::domains::income_tax::deduction::context::IncomeDeductionContext;
use crate::domains::income_tax::deduction::params::ExpenseDeductionParams;
use crate::domains::income_tax::deduction::types::{IncomeDeductionKind, IncomeDeductionLine};
use crate::error::JLawError;
use crate::types::amount::FinalAmount;

pub(crate) fn calculate_expense_deductions(
    ctx: &IncomeDeductionContext,
    _params: &ExpenseDeductionParams,
) -> Result<Vec<IncomeDeductionLine>, JLawError> {
    Ok(vec![IncomeDeductionLine {
        kind: IncomeDeductionKind::SocialInsurance,
        label: "社会保険料控除".into(),
        amount: FinalAmount::new(ctx.deductions.expense.social_insurance_premium_paid),
    }])
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::income_tax::deduction::context::{
        DependentDeductionInput, ExpenseDeductionInput, IncomeDeductionInput,
        PersonalDeductionInput,
    };
    use crate::domains::income_tax::deduction::params::SocialInsuranceDeductionParams;
    use crate::types::date::LegalDate;

    fn params() -> ExpenseDeductionParams {
        ExpenseDeductionParams {
            social_insurance: SocialInsuranceDeductionParams,
        }
    }

    fn ctx(social_insurance_premium_paid: u64) -> IncomeDeductionContext {
        IncomeDeductionContext {
            total_income_amount: 5_000_000,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid,
                },
            },
        }
    }

    #[test]
    fn social_insurance_deduction_uses_full_paid_amount() {
        let result = calculate_expense_deductions(&ctx(480_900), &params()).unwrap();
        assert_eq!(result[0].amount.as_yen(), 480_900);
    }

    #[test]
    fn social_insurance_deduction_allows_zero() {
        let result = calculate_expense_deductions(&ctx(0), &params()).unwrap();
        assert_eq!(result[0].amount.as_yen(), 0);
    }
}
