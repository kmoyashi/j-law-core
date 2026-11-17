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

fn manuscript_tax(payment_amount: u64, date: LegalDate) -> u64 {
    let params = load_withholding_tax_params(date).unwrap();
    calculate_withholding_tax(
        &ctx(
            payment_amount,
            0,
            date,
            WithholdingTaxCategory::ManuscriptAndLecture,
            HashSet::new(),
        ),
        &params,
    )
    .unwrap()
    .tax_amount
    .as_yen()
}

/// 国税庁「防衛特別所得税及び復興特別所得税（源泉徴収関係）Q&A」（令和8年5月）Q6 の計算例。
/// 2027年以後も合計税率（10.21% / 20.42%）は変わらない。
#[test]
fn defense_tax_era_keeps_combined_rate_nta_examples() {
    for date in [
        LegalDate::new(2027, 1, 1),
        LegalDate::new(2038, 1, 1),
        LegalDate::new(2047, 12, 31),
    ] {
        assert_eq!(manuscript_tax(222_222, date), 22_688);
        assert_eq!(manuscript_tax(1_333_333, date), 170_166);
    }
}

/// 復興特別所得税の課税期間（2047年まで）経過後は防衛特別所得税 1% のみが加算される。
#[test]
fn from_2048_only_defense_tax_is_added() {
    let date = LegalDate::new(2048, 1, 1);
    assert_eq!(manuscript_tax(100_000, date), 10_100);
    // 1,000,000 × 10.1% + 500,000 × 20.2%
    assert_eq!(manuscript_tax(1_500_000, date), 202_000);
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
