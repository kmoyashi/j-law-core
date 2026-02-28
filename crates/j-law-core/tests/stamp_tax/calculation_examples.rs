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

// ─── 非課税 ────────────────────────────────────────────────────────────────

/// 契約金額 9,999円（1万円未満・非課税）
#[test]
fn exempt_under_10k() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(9_999, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 0);
    assert!(!result.reduced_rate_applied);
}

// ─── 本則税額 ──────────────────────────────────────────────────────────────

/// 契約金額 50,000円（10万円以下・200円）
#[test]
fn bracket1_normal() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(50_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
}

/// 契約金額 300,000円（50万円以下・本則400円）
#[test]
fn bracket2_normal() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(300_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 400);
}

/// 契約金額 800,000円（100万円以下・本則1,000円）
#[test]
fn bracket3_normal() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(800_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 1_000);
}

/// 契約金額 3,000,000円（500万円以下・本則2,000円）
#[test]
fn bracket4_normal() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(3_000_000, (2024, 8, 1), false), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 2_000);
}

// ─── 軽減税額 ──────────────────────────────────────────────────────────────

/// 契約金額 300,000円（50万円以下・軽減200円）
#[test]
fn bracket2_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(300_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 200);
    assert!(result.reduced_rate_applied);
}

/// 契約金額 5,000,000円（500万円以下・軽減1,000円）
#[test]
fn bracket4_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(5_000_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 1_000);
    assert!(result.reduced_rate_applied);
}

/// 契約金額 50,000,000円（5,000万円以下・軽減10,000円）
#[test]
fn bracket6_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(50_000_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 10_000);
    assert!(result.reduced_rate_applied);
}

/// 契約金額 100,000,000円（1億円以下・軽減30,000円）
#[test]
fn bracket7_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(100_000_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 30_000);
    assert!(result.reduced_rate_applied);
}

/// 契約金額 10,000,000,000円（50億円超・軽減480,000円）
#[test]
fn bracket11_reduced() {
    let params = load_stamp_tax_params((2024, 8, 1)).unwrap();
    let result = calculate_stamp_tax(&ctx(10_000_000_000, (2024, 8, 1), true), &params).unwrap();
    assert_eq!(result.tax_amount.as_yen(), 480_000);
    assert!(result.reduced_rate_applied);
}
