#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::withholding_tax::{
    calculator::calculate_withholding_tax,
    context::{WithholdingTaxCategory, WithholdingTaxContext, WithholdingTaxFlag},
    policy::StandardWithholdingTaxPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_withholding_tax_params;

fn ctx(
    payment_amount: u64,
    separated_consumption_tax_amount: u64,
    date: LegalDate,
    category: WithholdingTaxCategory,
    is_submission_prize: bool,
) -> WithholdingTaxContext {
    let mut flags = HashSet::new();
    if is_submission_prize {
        flags.insert(WithholdingTaxFlag::IsSubmissionPrize);
    }
    WithholdingTaxContext {
        payment_amount,
        separated_consumption_tax_amount,
        category,
        target_date: date,
        flags,
        policy: Box::new(StandardWithholdingTaxPolicy),
    }
}

/// 原稿料・講演料等 100,000円。
///
/// 法的根拠: 国税庁タックスアンサー No.2795
/// 100万円以下の部分は 10.21%。
#[test]
fn manuscript_fee_100k() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let result = calculate_withholding_tax(
        &ctx(
            100_000,
            0,
            date,
            WithholdingTaxCategory::ManuscriptAndLecture,
            false,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.taxable_payment_amount.as_yen(), 100_000);
    assert_eq!(result.tax_amount.as_yen(), 10_210);
    assert_eq!(result.net_payment_amount.as_yen(), 89_790);
    assert!(!result.submission_prize_exempted);
    assert_eq!(result.breakdown.len(), 1);
}

/// 税理士等の報酬 1,500,000円。
///
/// 100万円以下 10.21%、超過部分 20.42% なので
/// 102,100円 + 102,100円 = 204,200円。
#[test]
fn professional_fee_1_5m() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let result = calculate_withholding_tax(
        &ctx(
            1_500_000,
            0,
            date,
            WithholdingTaxCategory::ProfessionalFee,
            false,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.taxable_payment_amount.as_yen(), 1_500_000);
    assert_eq!(result.tax_amount.as_yen(), 204_200);
    assert_eq!(result.net_payment_amount.as_yen(), 1_295_800);
    assert_eq!(result.breakdown.len(), 2);
}

/// 専属契約金 1,000,000円。
///
/// 国税庁タックスアンサー No.2810 の二段階税率に従う。
#[test]
fn exclusive_contract_fee_1m() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let result = calculate_withholding_tax(
        &ctx(
            1_000_000,
            0,
            date,
            WithholdingTaxCategory::ExclusiveContractFee,
            false,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.taxable_payment_amount.as_yen(), 1_000_000);
    assert_eq!(result.tax_amount.as_yen(), 102_100);
    assert_eq!(result.net_payment_amount.as_yen(), 897_900);
    assert_eq!(result.breakdown.len(), 1);
}

/// 請求書で消費税額 10,000円 が明示されている 110,000円の原稿料。
///
/// 源泉徴収税額の基礎は 100,000円として計算する。
#[test]
fn separated_consumption_tax_is_excluded_from_taxable_base() {
    let date = LegalDate::new(2026, 1, 1);
    let params = load_withholding_tax_params(date).unwrap();
    let result = calculate_withholding_tax(
        &ctx(
            110_000,
            10_000,
            date,
            WithholdingTaxCategory::ManuscriptAndLecture,
            false,
        ),
        &params,
    )
    .unwrap();

    assert_eq!(result.gross_payment_amount.as_yen(), 110_000);
    assert_eq!(result.taxable_payment_amount.as_yen(), 100_000);
    assert_eq!(result.tax_amount.as_yen(), 10_210);
    assert_eq!(result.net_payment_amount.as_yen(), 99_790);
}
