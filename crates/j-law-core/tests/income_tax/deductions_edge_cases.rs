//! 所得控除の境界値テスト

#![allow(clippy::disallowed_methods)]

use j_law_core::domains::income_tax::deduction::{
    calculate_income_deductions, BasicDeductionBracket, BasicDeductionParams,
    DependentDeductionInput, DependentDeductionParams, DonationDeductionParams,
    ExpenseDeductionInput, ExpenseDeductionParams, IncomeDeductionContext, IncomeDeductionInput,
    IncomeDeductionParams, LifeInsuranceDeductionBracket, LifeInsuranceDeductionParams,
    MedicalDeductionInput, MedicalDeductionParams, PersonalDeductionInput, PersonalDeductionParams,
    SocialInsuranceDeductionParams, SpouseDeductionInput, SpouseDeductionParams,
    SpouseIncomeBracket,
};
use j_law_core::{InputError, JLawError, LegalDate};

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

fn ctx(
    total_income_amount: u64,
    social_insurance_premium_paid: u64,
    spouse: Option<SpouseDeductionInput>,
) -> IncomeDeductionContext {
    IncomeDeductionContext {
        total_income_amount,
        target_date: LegalDate::new(2024, 1, 1),
        deductions: IncomeDeductionInput {
            personal: PersonalDeductionInput {
                spouse,
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
fn taxable_income_is_never_negative() {
    let result =
        calculate_income_deductions(&ctx(400_000, 30_000, None), &deduction_params_2024()).unwrap();

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
            calculate_income_deductions(&ctx(income, 0, None), &deduction_params_2024()).unwrap();
        assert_eq!(
            result.taxable_income_before_truncation.as_yen(),
            expected_before_truncation
        );
        assert_eq!(result.taxable_income.as_yen(), expected_taxable_income);
    }
}

#[test]
fn spouse_deduction_uses_taxpayer_income_thresholds() {
    let cases = [
        (9_000_000, 380_000),
        (9_000_001, 260_000),
        (9_500_001, 130_000),
        (10_000_001, 0),
    ];

    for (income, expected_spouse_deduction) in cases {
        let result = calculate_income_deductions(
            &ctx(
                income,
                0,
                Some(SpouseDeductionInput {
                    spouse_total_income_amount: 480_000,
                    is_same_household: true,
                    is_elderly: false,
                }),
            ),
            &deduction_params_2024(),
        )
        .unwrap();
        assert_eq!(
            result.breakdown[1].amount.as_yen(),
            expected_spouse_deduction
        );
    }
}

#[test]
fn medical_deduction_rejects_reimbursement_above_payment() {
    let result = calculate_income_deductions(
        &IncomeDeductionContext {
            total_income_amount: 5_000_000,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 0,
                    medical: Some(MedicalDeductionInput {
                        medical_expense_paid: 100_000,
                        reimbursed_amount: 100_001,
                    }),
                    life_insurance: None,
                    donation: None,
                },
            },
        },
        &deduction_params_2024(),
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidDeductionInput { .. }))
    ));
}
