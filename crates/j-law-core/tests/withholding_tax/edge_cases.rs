#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::withholding_tax::{
    calculator::calculate_withholding_tax,
    context::{WithholdingTaxCategory, WithholdingTaxContext, WithholdingTaxFlag},
    policy::StandardWithholdingTaxPolicy,
};
use j_law_core::{InputError, JLawError, LegalDate};
use j_law_registry::load_withholding_tax_params;

fn ctx(
    payment_amount: u64,
    separated_consumption_tax_amount: u64,
    date: LegalDate,
    category: WithholdingTaxCategory,
    flags: HashSet<WithholdingTaxFlag>,
) -> WithholdingTaxContext {
    WithholdingTaxContext {
        payment_amount,
        separated_consumption_tax_amount,
        category,
        target_date: date,
        flags,
        policy: Box::new(StandardWithholdingTaxPolicy),
    }
}

#[test]
fn submission_prize_under_or_equal_50k_is_exempt() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let mut flags = HashSet::new();
    flags.insert(WithholdingTaxFlag::IsSubmissionPrize);

    let result = calculate_withholding_tax(
        &ctx(
            50_000,
            0,
            date,
            WithholdingTaxCategory::ManuscriptAndLecture,
            flags,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
    assert_eq!(result.net_payment_amount.as_yen(), 50_000);
    assert!(result.submission_prize_exempted);
    assert!(result.breakdown.is_empty());
}

#[test]
fn submission_prize_over_50k_is_taxed_on_full_amount() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let mut flags = HashSet::new();
    flags.insert(WithholdingTaxFlag::IsSubmissionPrize);

    let result = calculate_withholding_tax(
        &ctx(
            50_001,
            0,
            date,
            WithholdingTaxCategory::ManuscriptAndLecture,
            flags,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 5_105);
    assert!(!result.submission_prize_exempted);
}

#[test]
fn date_before_2013_is_out_of_range() {
    let result = load_withholding_tax_params(LegalDate::new(2012, 12, 31));
    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::DateOutOfRange { .. }))
    ));
}

#[test]
fn date_after_2037_is_out_of_range() {
    let result = load_withholding_tax_params(LegalDate::new(2038, 1, 1));
    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::DateOutOfRange { .. }))
    ));
}

#[test]
fn separated_consumption_tax_cannot_exceed_payment_amount() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let result = calculate_withholding_tax(
        &ctx(
            100_000,
            100_001,
            date,
            WithholdingTaxCategory::ProfessionalFee,
            HashSet::new(),
        ),
        &params,
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidWithholdingInput { .. }))
    ));
}
