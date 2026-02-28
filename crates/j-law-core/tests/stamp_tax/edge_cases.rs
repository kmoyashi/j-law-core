#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_registry::load_stamp_tax_params;

fn ctx(amount: u64, date: (u16, u8, u8), reduced: bool) -> StampTaxContext {
    let mut flags = HashSet::new();
    if reduced {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }
    StampTaxContext {
        contract_amount: amount,
        target_date: date,
        flags,
        policy: Box::new(StandardNtaPolicy),
    }
}

// ─── 境界値テスト ──────────────────────────────────────────────────────────

/// 1万円ちょうど（非課税の境界を超えた最初の課税対象）
#[test]
fn edge_10k_exact() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(10_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
}

/// 9,999円（非課税の上限）
#[test]
fn edge_9999() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(9_999, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
}

/// 10万円ちょうど（10万円以下ブラケットの上限）
#[test]
fn edge_100k_exact() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(100_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
}

/// 100,001円（10万円超・50万円以下ブラケットに入る）
#[test]
fn edge_100k_plus1() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(100_001, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 400);
}

/// 10万円以下は軽減対象外（フラグありでも本則適用）
#[test]
fn edge_100k_reduced_flag_no_effect() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(100_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
    assert!(!result.reduced_rate_applied);
}

/// 100,001円 + 軽減フラグ → 軽減200円
#[test]
fn edge_100k_plus1_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(100_001, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
    assert!(result.reduced_rate_applied);
}

// ─── 軽減期間の境界テスト ──────────────────────────────────────────────────

/// 軽減期間の初日（2014/4/1）→ 軽減適用
#[test]
fn reduced_period_first_day() {
    let params = load_stamp_tax_params((2014, 4, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(5_000_000, (2014, 4, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 1_000);
    assert!(result.reduced_rate_applied);
}

/// 軽減期間の最終日（2027/3/31）→ 軽減適用
#[test]
fn reduced_period_last_day() {
    let params = load_stamp_tax_params((2027, 3, 31)).unwrap();
    let result = calculate_stamp_tax(&ctx(5_000_000, (2027, 3, 31), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 1_000);
    assert!(result.reduced_rate_applied);
}

/// 軽減期間の翌日（2027/4/1）→ 本則適用
#[test]
fn reduced_period_day_after() {
    let params = load_stamp_tax_params((2027, 4, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(5_000_000, (2027, 4, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 2_000);
    assert!(!result.reduced_rate_applied);
}

/// 契約金額 0円
#[test]
fn zero_amount() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(0, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
}
