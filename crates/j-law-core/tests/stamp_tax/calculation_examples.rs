#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxDocumentCode, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_core::LegalDate;
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
fn article1_real_estate_base_rate() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article1OtherTransfer,
            Some(5_000_000),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 2_000);
    assert_eq!(result.rule_label, "100万円を超え500万円以下のもの");
    assert!(result.applied_special_rule.is_none());
}

#[test]
fn article1_real_estate_reduced_rate() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let reduced = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article1RealEstateTransfer,
            Some(50_000_000),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(reduced.tax_amount.as_yen(), 10_000);
    assert_eq!(
        reduced.applied_special_rule.as_deref(),
        Some("article1_real_estate_transfer_reduced")
    );
}

#[test]
fn article2_construction_reduced_rate() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article2ConstructionWork,
            Some(2_500_000),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 500);
    assert_eq!(
        result.applied_special_rule.as_deref(),
        Some("article2_construction_work_reduced")
    );
}

#[test]
fn article3_special_flat_rate() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article3BillSpecialFlat200,
            None,
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 200);
    assert_eq!(result.rule_label, "200円");
}

#[test]
fn article5_fixed_document_tax() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article5MergerOrSplit,
            None,
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 40_000);
}

#[test]
fn article15_assignment_amount_and_no_amount() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let amount_result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article15AssignmentOrAssumption,
            Some(10_000),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(amount_result.tax_amount.as_yen(), 200);

    let no_amount_result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article15AssignmentOrAssumption,
            None,
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(no_amount_result.tax_amount.as_yen(), 200);
    assert_eq!(no_amount_result.rule_label, "契約金額の記載のないもの");
}

#[test]
fn article16_dividend_threshold() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article16DividendReceipt,
            Some(3_000),
            LegalDate::new(2024, 8, 1),
            &[],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 200);
}

#[test]
fn article17_non_business_exempt() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article17SalesReceipt,
            Some(70_000),
            LegalDate::new(2024, 8, 1),
            &[StampTaxFlag::Article17NonBusinessExempt],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
    assert_eq!(
        result.applied_special_rule.as_deref(),
        Some("article17_non_business_exempt")
    );
}

#[test]
fn article18_passbook_exempt() {
    let params = load_stamp_tax_params(LegalDate::new(2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(
        &ctx(
            StampTaxDocumentCode::Article18Passbook,
            None,
            LegalDate::new(2024, 8, 1),
            &[StampTaxFlag::Article18TaxReserveDepositPassbook],
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.tax_amount.as_yen(), 0);
    assert_eq!(
        result.applied_special_rule.as_deref(),
        Some("article18_tax_reserve_deposit_passbook")
    );
}
