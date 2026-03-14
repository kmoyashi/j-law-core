#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxDocumentCode, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_core::{InputError, JLawError, LegalDate};
use j_law_registry::load_stamp_tax_params;

fn ctx(
    document_code: StampTaxDocumentCode,
    stated_amount: Option<u64>,
    date: LegalDate,
    flags: &[StampTaxFlag],
) -> StampTaxContext {
    let mut set = HashSet::new();
    for flag in flags {
        set.insert(*flag);
    }

    StampTaxContext {
        document_code,
        stated_amount,
        target_date: date,
        flags: set,
        policy: Box::new(StandardNtaPolicy),
    }
}

#[test]
fn article1_under_10k_is_non_taxable() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article1OtherTransfer,
            Some(9_999),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
}

#[test]
fn article3_no_amount_is_non_taxable() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article3BillAmountTable,
            None,
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
}

#[test]
fn article4_requires_amount() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article4SecurityCertificate,
            None,
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidStampTaxInput { .. }))
    ));
}

#[test]
fn fixed_document_rejects_amount() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article20SealBook,
            Some(1),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidStampTaxInput { .. }))
    ));
}

#[test]
fn invalid_flag_for_document_is_rejected() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article5MergerOrSplit,
            None,
            LegalDate::new(2024, 8, 1),
            &[StampTaxFlag::Article17NonBusinessExempt],
        ),
        &params,
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidStampTaxInput { .. }))
    ));
}

#[test]
fn amount_required_for_article8_small_deposit_flag() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article8DepositCertificate,
            None,
            LegalDate::new(2024, 8, 1),
            &[StampTaxFlag::Article8SmallDepositExempt],
        ),
        &params,
    );

    assert!(matches!(
        result,
        Err(JLawError::Input(InputError::InvalidStampTaxInput { .. }))
    ));
}

#[test]
fn article8_small_deposit_exempt_applies_below_threshold() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article8DepositCertificate,
            Some(9_999),
            LegalDate::new(2024, 8, 1),
            &[StampTaxFlag::Article8SmallDepositExempt],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
}

#[test]
fn article2_reduction_ends_after_2027_03_31() {
    let params = load_stamp_tax_params(LegalDate::new(2027, 4, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article2ConstructionWork,
            Some(2_500_000),
            LegalDate::new(2027, 4, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 1_000);
    assert!(result.applied_special_rule.is_none());
}
