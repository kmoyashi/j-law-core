//! Registry 連携の通しテスト

#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculate_income_tax_assessment, DependentDeductionInput, DonationDeductionInput,
    ExpenseDeductionInput, IncomeDeductionContext, IncomeDeductionInput,
    IncomeTaxAssessmentContext, IncomeTaxFlag, LifeInsuranceDeductionInput, MedicalDeductionInput,
    PersonalDeductionInput, StandardIncomeTaxPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::{load_income_tax_deduction_params, load_income_tax_params};

#[test]
fn registry_loaded_deductions_flow_into_income_tax_assessment() {
    let mut flags = HashSet::new();
    flags.insert(IncomeTaxFlag::ApplyReconstructionTax);

    let ctx = IncomeTaxAssessmentContext {
        deduction_context: IncomeDeductionContext {
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
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let deduction_params = load_income_tax_deduction_params(LegalDate::new(2024, 1, 1)).unwrap();
    let tax_params = load_income_tax_params(LegalDate::new(2024, 1, 1)).unwrap();
    let result = calculate_income_tax_assessment(&ctx, &deduction_params, &tax_params).unwrap();

    assert_eq!(result.deductions.total_deductions.as_yen(), 1_593_000);
    assert_eq!(result.deductions.taxable_income.as_yen(), 4_407_000);
    assert_eq!(result.tax.base_tax.as_yen(), 453_900);
    assert_eq!(result.tax.reconstruction_tax.as_yen(), 9_531);
    assert_eq!(result.tax.total_tax.as_yen(), 463_400);
}
