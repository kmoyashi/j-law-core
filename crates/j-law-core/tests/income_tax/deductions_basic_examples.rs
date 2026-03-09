//! 所得控除の基本計算例テスト
//!
//! 出典:
//! - 国税庁「基礎控除とは」（令和6年分）
//! - 国税庁「手順3 所得から差し引かれる金額（所得控除）を計算する / 社会保険料控除」

#![allow(clippy::disallowed_methods)]

use j_law_core::domains::income_tax::deduction::{
    calculate_income_deductions, BasicDeductionBracket, BasicDeductionParams,
    ExpenseDeductionInput, ExpenseDeductionParams, IncomeDeductionContext, IncomeDeductionInput,
    IncomeDeductionKind, IncomeDeductionParams, PersonalDeductionInput, PersonalDeductionParams,
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
fn basic_and_social_insurance_deductions_are_aggregated() {
    let result =
        calculate_income_deductions(&ctx(5_480_900, 480_900), &deduction_params_2024()).unwrap();

    assert_eq!(result.total_income_amount.as_yen(), 5_480_900);
    assert_eq!(result.total_deductions.as_yen(), 960_900);
    assert_eq!(result.taxable_income_before_truncation.as_yen(), 4_520_000);
    assert_eq!(result.taxable_income.as_yen(), 4_520_000);
    assert_eq!(result.breakdown.len(), 2);
    assert_eq!(result.breakdown[0].kind, IncomeDeductionKind::Basic);
    assert_eq!(result.breakdown[0].label, "基礎控除");
    assert_eq!(result.breakdown[0].amount.as_yen(), 480_000);
    assert_eq!(
        result.breakdown[1].kind,
        IncomeDeductionKind::SocialInsurance
    );
    assert_eq!(result.breakdown[1].label, "社会保険料控除");
    assert_eq!(result.breakdown[1].amount.as_yen(), 480_900);
}

#[test]
fn basic_deduction_uses_2024_income_thresholds() {
    let cases = [
        (24_000_000, 480_000),
        (24_000_001, 320_000),
        (24_500_001, 160_000),
        (25_000_001, 0),
    ];

    for (income, expected_basic_deduction) in cases {
        let result =
            calculate_income_deductions(&ctx(income, 0), &deduction_params_2024()).unwrap();
        assert_eq!(
            result.breakdown[0].amount.as_yen(),
            expected_basic_deduction
        );
    }
}
