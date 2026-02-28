//! 境界値テスト
//!
//! 200万円・400万円・800万円の各境界と、その前後1円での挙動を検証する。

#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_registry::load_brokerage_fee_params;

fn ctx(price: u64) -> RealEstateContext {
    RealEstateContext {
        price,
        target_date: (2024, 8, 1),
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    }
}

fn ctx_flag(price: u64, flag: RealEstateFlag) -> RealEstateContext {
    let mut flags = HashSet::new();
    flags.insert(flag);
    RealEstateContext {
        price,
        target_date: (2024, 8, 1),
        flags,
        policy: Box::new(StandardMliitPolicy),
    }
}

// ─── 200万円境界 ──────────────────────────────────────────────────────────────

/// 売買価格 1円（最小値）→ tier1 のみ
#[test]
fn edge_price_1yen() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(1), &params).unwrap();
    // 1 × 5/100 = 0.05 → Floor → 0円
    assert_eq!(result.total_without_tax.as_yen(), 0);
    assert_eq!(result.breakdown.len(), 1);
}

/// 売買価格 2,000,000円（tier1 上限・ちょうど）
#[test]
fn edge_price_2m_exact() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(2_000_000), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 100_000);
    assert_eq!(result.breakdown.len(), 1); // tier1 のみ
}

/// 売買価格 2,000,001円（tier2 開始・tier1 上限の1円上）
#[test]
fn edge_price_2m_plus_1() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(2_000_001), &params).unwrap();
    // tier1: 2,000,000 × 5% = 100,000
    // tier2: 1 × 4/100 = 0.04 → Floor → 0
    assert_eq!(result.total_without_tax.as_yen(), 100_000);
    assert_eq!(result.breakdown.len(), 2); // tier1 + tier2
}

// ─── 400万円境界 ──────────────────────────────────────────────────────────────

/// 売買価格 4,000,000円（tier2 上限・ちょうど）
#[test]
fn edge_price_4m_exact() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(4_000_000), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    assert_eq!(result.breakdown.len(), 2); // tier1 + tier2
}

/// 売買価格 4,000,001円（tier3 開始）
#[test]
fn edge_price_4m_plus_1() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(4_000_001), &params).unwrap();
    // tier1: 100,000 / tier2: 80,000 / tier3: 1 × 3/100 = 0 (Floor)
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    assert_eq!(result.breakdown.len(), 3); // tier1 + tier2 + tier3
}

// ─── 800万円境界（低廉特例） ──────────────────────────────────────────────────

/// 売買価格 8,000,000円・フラグあり（特例上限ちょうど）
#[test]
fn edge_low_cost_8m_with_flag() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flag(8_000_000, RealEstateFlag::IsLowCostVacantHouse),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 330_000);
    assert!(result.low_cost_special_applied);
}

/// 売買価格 8,000,000円・フラグなし（特例不適用）
#[test]
fn edge_low_cost_8m_without_flag() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(8_000_000), &params).unwrap();
    assert!(!result.low_cost_special_applied);
    // 通常3段階計算:
    // tier1: 2,000,000 × 5% = 100,000
    // tier2: 2,000,000 × 4% = 80,000
    // tier3: 4,000,000 × 3% = 120,000
    // 合計: 300,000
    assert_eq!(result.total_without_tax.as_yen(), 300_000);
}

/// 売買価格 8,000,001円・フラグあり（特例適用外・価格が上限を1円超過）
#[test]
fn edge_low_cost_8m_plus_1_with_flag() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flag(8_000_001, RealEstateFlag::IsLowCostVacantHouse),
        &params,
    )
    .unwrap();
    assert!(!result.low_cost_special_applied);
}
