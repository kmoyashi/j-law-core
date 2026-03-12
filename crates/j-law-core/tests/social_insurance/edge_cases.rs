#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::social_insurance::{
    calculate_social_insurance_premium, SocialInsuranceContext, SocialInsuranceFlag,
    SocialInsurancePrefecture, StandardNenkinPolicy,
};
use j_law_core::{CalculationError, JLawError, LegalDate};
use j_law_registry::load_social_insurance_params;

fn ctx(
    standard_monthly_remuneration: u64,
    date: LegalDate,
    prefecture: SocialInsurancePrefecture,
    care: bool,
) -> SocialInsuranceContext {
    let mut flags = HashSet::new();
    if care {
        flags.insert(SocialInsuranceFlag::IsCareInsuranceApplicable);
    }
    SocialInsuranceContext {
        standard_monthly_remuneration,
        target_date: date,
        prefecture,
        flags,
        policy: Box::new(StandardNenkinPolicy),
    }
}

#[test]
fn tokyo_2025_rate_is_loaded() {
    let params = load_social_insurance_params(LegalDate::new(2025, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(
            58_000,
            LegalDate::new(2025, 3, 1),
            SocialInsurancePrefecture::Tokyo,
            false,
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 2_874);
    assert_eq!(result.pension_amount.as_yen(), 5_307);
    assert_eq!(result.total_amount.as_yen(), 8_181);
}

#[test]
fn exact_half_yen_in_combined_health_and_care_is_floored_for_payroll_deduction() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(
            150_000,
            LegalDate::new(2026, 3, 1),
            SocialInsurancePrefecture::Tokyo,
            true,
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 8_602);
    assert_eq!(result.total_amount.as_yen(), 22_327);
}

#[test]
fn pension_cap_applies_above_650k() {
    let params = load_social_insurance_params(LegalDate::new(2025, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(
            680_000,
            LegalDate::new(2025, 3, 1),
            SocialInsurancePrefecture::Tokyo,
            false,
        ),
        &params,
    )
    .unwrap();
    assert_eq!(
        result.health_standard_monthly_remuneration.as_yen(),
        680_000
    );
    assert_eq!(
        result.pension_standard_monthly_remuneration.as_yen(),
        650_000
    );
    assert_eq!(result.health_related_amount.as_yen(), 33_694);
    assert_eq!(result.pension_amount.as_yen(), 59_475);
}

#[test]
fn invalid_standard_monthly_remuneration_returns_error() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(
            305_000,
            LegalDate::new(2026, 3, 1),
            SocialInsurancePrefecture::Tokyo,
            false,
        ),
        &params,
    );
    assert!(matches!(
        result,
        Err(JLawError::Calculation(
            CalculationError::PolicyNotApplicable { .. }
        ))
    ));
}
