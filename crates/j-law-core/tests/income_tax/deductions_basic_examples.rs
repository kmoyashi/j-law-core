//! 所得控除の基本計算例テスト
//!
//! 出典:
//! - 国税庁「基礎控除とは」（令和6年分）
//! - 国税庁「手順3 所得から差し引かれる金額（所得控除）を計算する / 社会保険料控除」
//! - 国税庁「医療費を支払ったとき（医療費控除）」
//! - 国税庁「生命保険料控除」
//! - 国税庁「寄附金を支出したとき」

#![allow(clippy::disallowed_methods)]

use j_law_core::domains::income_tax::deduction::{
    calculate_income_deductions, BasicDeductionBracket, BasicDeductionParams,
    DependentDeductionInput, DependentDeductionParams, DonationDeductionInput,
    DonationDeductionParams, ExpenseDeductionInput, ExpenseDeductionParams, IncomeDeductionContext,
    IncomeDeductionInput, IncomeDeductionKind, IncomeDeductionParams,
    LifeInsuranceDeductionBracket, LifeInsuranceDeductionInput, LifeInsuranceDeductionParams,
    MedicalDeductionInput, MedicalDeductionParams, PersonalDeductionInput, PersonalDeductionParams,
    SocialInsuranceDeductionParams, SpouseDeductionInput, SpouseDeductionParams,
    SpouseIncomeBracket,
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
        },
    }
}

fn ctx(
    total_income_amount: u64,
    social_insurance_premium_paid: u64,
    spouse: Option<SpouseDeductionInput>,
    dependent: DependentDeductionInput,
) -> IncomeDeductionContext {
    IncomeDeductionContext {
        total_income_amount,
        target_date: LegalDate::new(2024, 1, 1),
        deductions: IncomeDeductionInput {
            personal: PersonalDeductionInput { spouse, dependent },
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
fn basic_and_social_insurance_deductions_are_aggregated() {
    let result = calculate_income_deductions(
        &ctx(5_480_900, 480_900, None, DependentDeductionInput::default()),
        &deduction_params_2024(),
    )
    .unwrap();

    assert_eq!(result.total_income_amount.as_yen(), 5_480_900);
    assert_eq!(result.total_deductions.as_yen(), 960_900);
    assert_eq!(result.taxable_income_before_truncation.as_yen(), 4_520_000);
    assert_eq!(result.taxable_income.as_yen(), 4_520_000);
    assert_eq!(result.breakdown.len(), 7);
    assert_eq!(result.breakdown[0].kind, IncomeDeductionKind::Basic);
    assert_eq!(result.breakdown[0].label, "基礎控除");
    assert_eq!(result.breakdown[0].amount.as_yen(), 480_000);
    assert_eq!(result.breakdown[1].kind, IncomeDeductionKind::Spouse);
    assert_eq!(result.breakdown[1].amount.as_yen(), 0);
    assert_eq!(result.breakdown[2].kind, IncomeDeductionKind::Dependent);
    assert_eq!(result.breakdown[2].amount.as_yen(), 0);
    assert_eq!(
        result.breakdown[3].kind,
        IncomeDeductionKind::SocialInsurance
    );
    assert_eq!(result.breakdown[3].label, "社会保険料控除");
    assert_eq!(result.breakdown[3].amount.as_yen(), 480_900);
    assert_eq!(result.breakdown[4].kind, IncomeDeductionKind::Medical);
    assert_eq!(result.breakdown[5].kind, IncomeDeductionKind::LifeInsurance);
    assert_eq!(result.breakdown[6].kind, IncomeDeductionKind::Donation);
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
        let result = calculate_income_deductions(
            &ctx(income, 0, None, DependentDeductionInput::default()),
            &deduction_params_2024(),
        )
        .unwrap();
        assert_eq!(
            result.breakdown[0].amount.as_yen(),
            expected_basic_deduction
        );
    }
}

#[test]
fn spouse_and_dependent_deductions_are_calculated() {
    let result = calculate_income_deductions(
        &ctx(
            8_000_000,
            200_000,
            Some(SpouseDeductionInput {
                spouse_total_income_amount: 480_000,
                is_same_household: true,
                is_elderly: true,
            }),
            DependentDeductionInput {
                general_count: 1,
                specific_count: 1,
                elderly_cohabiting_count: 1,
                elderly_other_count: 1,
            },
        ),
        &deduction_params_2024(),
    )
    .unwrap();

    assert_eq!(result.breakdown[1].amount.as_yen(), 480_000);
    assert_eq!(result.breakdown[2].amount.as_yen(), 2_070_000);
    assert_eq!(result.total_deductions.as_yen(), 3_230_000);
    assert_eq!(result.taxable_income.as_yen(), 4_770_000);
}

#[test]
fn medical_life_insurance_and_donation_deductions_are_calculated() {
    let result = calculate_income_deductions(
        &IncomeDeductionContext {
            total_income_amount: 6_000_000,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 150_000,
                    medical: Some(MedicalDeductionInput {
                        medical_expense_paid: 500_000,
                        reimbursed_amount: 50_000,
                    }),
                    life_insurance: Some(LifeInsuranceDeductionInput {
                        new_general_paid_amount: 100_000,
                        new_individual_pension_paid_amount: 60_000,
                        new_care_medical_paid_amount: 80_000,
                        old_general_paid_amount: 0,
                        old_individual_pension_paid_amount: 0,
                    }),
                    donation: Some(DonationDeductionInput {
                        qualified_donation_amount: 500_000,
                    }),
                },
            },
        },
        &deduction_params_2024(),
    )
    .unwrap();

    assert_eq!(result.breakdown[4].amount.as_yen(), 350_000);
    assert_eq!(result.breakdown[5].amount.as_yen(), 115_000);
    assert_eq!(result.breakdown[6].amount.as_yen(), 498_000);
    assert_eq!(result.total_deductions.as_yen(), 1_593_000);
    assert_eq!(result.taxable_income.as_yen(), 4_407_000);
}
