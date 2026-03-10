//! 所得控除から所得税額までの通し計算テスト

#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculate_income_tax_assessment, deduction::BasicDeductionBracket,
    deduction::BasicDeductionParams, deduction::DependentDeductionInput,
    deduction::DependentDeductionParams, deduction::DonationDeductionParams,
    deduction::ExpenseDeductionInput, deduction::ExpenseDeductionParams,
    deduction::IncomeDeductionContext, deduction::IncomeDeductionInput,
    deduction::IncomeDeductionParams, deduction::LifeInsuranceDeductionBracket,
    deduction::LifeInsuranceDeductionParams, deduction::MedicalDeductionParams,
    deduction::PersonalDeductionInput, deduction::PersonalDeductionParams,
    deduction::SocialInsuranceDeductionParams, deduction::SpouseDeductionInput,
    deduction::SpouseDeductionParams, deduction::SpouseIncomeBracket, IncomeTaxAssessmentContext,
    IncomeTaxBracket, IncomeTaxFlag, IncomeTaxParams, ReconstructionTaxParams,
    StandardIncomeTaxPolicy,
};
use j_law_core::LegalDate;

fn tax_params_2024() -> IncomeTaxParams {
    IncomeTaxParams {
        brackets: vec![
            IncomeTaxBracket {
                label: "195万円以下".into(),
                income_from: 0,
                income_to_inclusive: Some(1_950_000),
                rate_numer: 5,
                rate_denom: 100,
                deduction: 0,
            },
            IncomeTaxBracket {
                label: "195万円超330万円以下".into(),
                income_from: 1_950_001,
                income_to_inclusive: Some(3_300_000),
                rate_numer: 10,
                rate_denom: 100,
                deduction: 97_500,
            },
            IncomeTaxBracket {
                label: "330万円超695万円以下".into(),
                income_from: 3_300_001,
                income_to_inclusive: Some(6_950_000),
                rate_numer: 20,
                rate_denom: 100,
                deduction: 427_500,
            },
            IncomeTaxBracket {
                label: "695万円超900万円以下".into(),
                income_from: 6_950_001,
                income_to_inclusive: Some(9_000_000),
                rate_numer: 23,
                rate_denom: 100,
                deduction: 636_000,
            },
            IncomeTaxBracket {
                label: "900万円超1800万円以下".into(),
                income_from: 9_000_001,
                income_to_inclusive: Some(18_000_000),
                rate_numer: 33,
                rate_denom: 100,
                deduction: 1_536_000,
            },
            IncomeTaxBracket {
                label: "1800万円超4000万円以下".into(),
                income_from: 18_000_001,
                income_to_inclusive: Some(40_000_000),
                rate_numer: 40,
                rate_denom: 100,
                deduction: 2_796_000,
            },
            IncomeTaxBracket {
                label: "4000万円超".into(),
                income_from: 40_000_001,
                income_to_inclusive: None,
                rate_numer: 45,
                rate_denom: 100,
                deduction: 4_796_000,
            },
        ],
        reconstruction_tax: Some(ReconstructionTaxParams {
            rate_numer: 21,
            rate_denom: 1000,
            effective_from_year: 2013,
            effective_to_year_inclusive: 2037,
        }),
    }
}

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

#[test]
fn assessment_connects_deductions_to_income_tax() {
    let mut flags = HashSet::new();
    flags.insert(IncomeTaxFlag::ApplyReconstructionTax);

    let ctx = IncomeTaxAssessmentContext {
        deduction_context: IncomeDeductionContext {
            total_income_amount: 5_480_900,
            target_date: LegalDate::new(2024, 1, 1),
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: Some(SpouseDeductionInput {
                        spouse_total_income_amount: 480_000,
                        is_same_household: true,
                        is_elderly: false,
                    }),
                    dependent: DependentDeductionInput {
                        general_count: 1,
                        specific_count: 0,
                        elderly_cohabiting_count: 0,
                        elderly_other_count: 0,
                    },
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 480_900,
                    medical: None,
                    life_insurance: None,
                    donation: None,
                },
            },
        },
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let result =
        calculate_income_tax_assessment(&ctx, &deduction_params_2024(), &tax_params_2024())
            .unwrap();

    assert_eq!(result.deductions.taxable_income.as_yen(), 3_760_000);
    assert_eq!(result.tax.base_tax.as_yen(), 324_500);
    assert_eq!(result.tax.reconstruction_tax.as_yen(), 6_814);
    assert_eq!(result.tax.total_tax.as_yen(), 331_300);
}
