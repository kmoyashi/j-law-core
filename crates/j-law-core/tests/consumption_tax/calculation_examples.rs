//! 消費税計算例テスト
//!
//! 出典: 財務省「消費税法第29条」・国税庁「消費税の税率」
//! https://www.nta.go.jp/taxes/shiraberu/taxanswer/shohi/6303.htm

#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_consumption_tax_params;

fn ctx(amount: u64, date: LegalDate) -> ConsumptionTaxContext {
    ConsumptionTaxContext {
        amount,
        target_date: date,
        flags: HashSet::new(),
        policy: Box::new(StandardConsumptionTaxPolicy),
    }
}

fn ctx_reduced(amount: u64, date: LegalDate) -> ConsumptionTaxContext {
    let mut flags = HashSet::new();
    flags.insert(ConsumptionTaxFlag::ReducedRate);
    ConsumptionTaxContext {
        amount,
        target_date: date,
        flags,
        policy: Box::new(StandardConsumptionTaxPolicy),
    }
}

// ─── 各税率の基本計算 ────────────────────────────────────────────────────────

/// 消費税3%（1990年1月1日）
/// 100,000円 × 3% = 3,000円 / 税込 103,000円
#[test]
fn rate_3pct_standard() {
    let params = load_consumption_tax_params(LegalDate::new(1990, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1990, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 3_000);
    assert_eq!(result.amount_with_tax.as_yen(), 103_000);
    assert_eq!(result.amount_without_tax.as_yen(), 100_000);
    assert_eq!(result.applied_rate_numer, 3);
    assert_eq!(result.applied_rate_denom, 100);
    assert!(!result.is_reduced_rate);
}

/// 消費税5%（2000年1月1日）
/// 100,000円 × 5% = 5,000円 / 税込 105,000円
#[test]
fn rate_5pct_standard() {
    let params = load_consumption_tax_params(LegalDate::new(2000, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2000, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 5_000);
    assert_eq!(result.amount_with_tax.as_yen(), 105_000);
    assert_eq!(result.applied_rate_numer, 5);
}

/// 消費税8%（2016年1月1日）
/// 100,000円 × 8% = 8,000円 / 税込 108,000円
#[test]
fn rate_8pct_standard() {
    let params = load_consumption_tax_params(LegalDate::new(2016, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2016, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 8_000);
    assert_eq!(result.amount_with_tax.as_yen(), 108_000);
    assert_eq!(result.applied_rate_numer, 8);
}

/// 消費税10%（2020年1月1日・標準税率）
/// 100,000円 × 10% = 10,000円 / 税込 110,000円
#[test]
fn rate_10pct_standard() {
    let params = load_consumption_tax_params(LegalDate::new(2020, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2020, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 10_000);
    assert_eq!(result.amount_with_tax.as_yen(), 110_000);
    assert_eq!(result.applied_rate_numer, 10);
    assert!(!result.is_reduced_rate);
}

/// 消費税8%軽減税率（2020年1月1日・飲食料品等）
/// 100,000円 × 8% = 8,000円 / 税込 108,000円
#[test]
fn rate_8pct_reduced() {
    let params = load_consumption_tax_params(LegalDate::new(2020, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx_reduced(100_000, LegalDate::new(2020, 1, 1)), &params)
            .unwrap();
    assert_eq!(result.tax_amount.as_yen(), 8_000);
    assert_eq!(result.amount_with_tax.as_yen(), 108_000);
    assert_eq!(result.applied_rate_numer, 8);
    assert!(result.is_reduced_rate);
}

// ─── 境界値テスト ────────────────────────────────────────────────────────────

/// 1989-04-01（消費税導入初日）は3%
#[test]
fn boundary_1989_04_01_is_3pct() {
    let params = load_consumption_tax_params(LegalDate::new(1989, 4, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1989, 4, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 3_000);
    assert_eq!(result.applied_rate_numer, 3);
}

/// 1997-03-31（消費税3%最終日）は3%
#[test]
fn boundary_1997_03_31_is_3pct() {
    let params = load_consumption_tax_params(LegalDate::new(1997, 3, 31)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1997, 3, 31)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 3_000);
}

/// 1997-04-01（消費税5%施行日）は5%
#[test]
fn boundary_1997_04_01_is_5pct() {
    let params = load_consumption_tax_params(LegalDate::new(1997, 4, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1997, 4, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 5_000);
}

/// 2014-04-01（消費税8%施行日）は8%
#[test]
fn boundary_2014_04_01_is_8pct() {
    let params = load_consumption_tax_params(LegalDate::new(2014, 4, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2014, 4, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 8_000);
}

/// 2019-09-30（消費税8%最終日）は8%・軽減税率なし
#[test]
fn boundary_2019_09_30_is_8pct_no_reduced() {
    let params = load_consumption_tax_params(LegalDate::new(2019, 9, 30)).unwrap();
    assert!(params.reduced_rate.is_none());
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2019, 9, 30)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 8_000);
}

/// 2019-10-01（消費税10%施行日・標準税率）は10%
#[test]
fn boundary_2019_10_01_standard_is_10pct() {
    let params = load_consumption_tax_params(LegalDate::new(2019, 10, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(2019, 10, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 10_000);
}

/// 2019-10-01（消費税10%施行日・軽減税率）は8%
#[test]
fn boundary_2019_10_01_reduced_is_8pct() {
    let params = load_consumption_tax_params(LegalDate::new(2019, 10, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx_reduced(100_000, LegalDate::new(2019, 10, 1)), &params)
            .unwrap();
    assert_eq!(result.tax_amount.as_yen(), 8_000);
    assert!(result.is_reduced_rate);
}

// ─── 消費税導入前（0%）────────────────────────────────────────────────────────

/// 消費税導入前（1989-03-31以前）はエラーではなく税額0を返す
/// 100,000円 × 0% = 0円 / 税込 100,000円
#[test]
fn before_introduction_returns_zero_tax() {
    let params = load_consumption_tax_params(LegalDate::new(1989, 3, 31)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1989, 3, 31)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
    assert_eq!(result.amount_with_tax.as_yen(), 100_000);
    assert_eq!(result.applied_rate_numer, 0);
}

/// 大昔の日付でも税額0を返す（エラーにならない）
#[test]
fn very_old_date_returns_zero_tax() {
    let params = load_consumption_tax_params(LegalDate::new(1970, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_000, LegalDate::new(1970, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
}

// ─── 端数処理テスト ──────────────────────────────────────────────────────────

/// 端数切り捨て（消費税法第45条）
/// 100,001円 × 10% = 10,000.1円 → 切り捨て → 10,000円
#[test]
fn floor_rounding_10pct() {
    let params = load_consumption_tax_params(LegalDate::new(2020, 1, 1)).unwrap();
    let result =
        calculate_consumption_tax(&ctx(100_001, LegalDate::new(2020, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 10_000);
    assert_eq!(result.amount_with_tax.as_yen(), 110_001);
}

/// 課税標準額0円の場合は税額0
#[test]
fn zero_amount_zero_tax() {
    let params = load_consumption_tax_params(LegalDate::new(2020, 1, 1)).unwrap();
    let result = calculate_consumption_tax(&ctx(0, LegalDate::new(2020, 1, 1)), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
    assert_eq!(result.amount_with_tax.as_yen(), 0);
}
