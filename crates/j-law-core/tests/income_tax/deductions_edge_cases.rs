//! 所得控除の境界値テスト

#![allow(clippy::disallowed_methods)]

use j_law_core::domains::income_tax::deduction::{
    calculate_income_deductions, BasicDeductionBracket, BasicDeductionParams,
    ExpenseDeductionInput, ExpenseDeductionParams, IncomeDeductionContext, IncomeDeductionInput,
    IncomeDeductionParams, PersonalDeductionInput, PersonalDeductionParams,
    SocialInsuranceDeductionParams,
};
use j_law_core::LegalDate;

fn deduction_params_2024() -> IncomeDeductionParams {
    IncomeDeductionParams {
        personal: PersonalDeductionParams {
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
        },
        expense: ExpenseDeductionParams {
            social_insurance: SocialInsuranceDeductionParams,
        },
    }
}

fn ctx(total_income_amount: u64, social_insurance_premium_paid: u64) -> IncomeDeductionContext {
    IncomeDeductionContext {
        total_income_amount,
        target_date: LegalDate::new(2024, 1, 1),
        deductions: IncomeDeductionInput {
            personal: PersonalDeductionInput {},
            expense: ExpenseDeductionInput {
                social_insurance_premium_paid,
            },
        },
    }
}

#[test]
fn taxable_income_is_never_negative() {
    let result =
        calculate_income_deductions(&ctx(400_000, 30_000), &deduction_params_2024()).unwrap();

    assert_eq!(result.total_deductions.as_yen(), 510_000);
    assert_eq!(result.taxable_income_before_truncation.as_yen(), 0);
    assert_eq!(result.taxable_income.as_yen(), 0);
}

#[test]
fn taxable_income_truncates_below_1000() {
    let cases = [
        (480_999, 999, 0),
        (481_000, 1_000, 1_000),
        (481_999, 1_999, 1_000),
    ];

    for (income, expected_before_truncation, expected_taxable_income) in cases {
        let result =
            calculate_income_deductions(&ctx(income, 0), &deduction_params_2024()).unwrap();
        assert_eq!(
            result.taxable_income_before_truncation.as_yen(),
            expected_before_truncation
        );
        assert_eq!(result.taxable_income.as_yen(), expected_taxable_income);
    }
}
