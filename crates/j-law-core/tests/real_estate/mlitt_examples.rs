//! 国土交通省公式計算例テスト
//!
//! 出典: 国土交通省「宅地建物取引業法の解釈・運用の考え方」第46条関係
//! https://www.mlit.go.jp/totikensangyo/const/1_6_bt_000083.html

#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_registry::load_brokerage_fee_params;

fn ctx(price: u64, date: (u16, u8, u8)) -> RealEstateContext {
    RealEstateContext {
        price,
        target_date: date,
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    }
}

fn ctx_with_flag(price: u64, date: (u16, u8, u8), flag: RealEstateFlag) -> RealEstateContext {
    let mut flags = HashSet::new();
    flags.insert(flag);
    RealEstateContext {
        price,
        target_date: date,
        flags,
        policy: Box::new(StandardMliitPolicy),
    }
}

// ─── 2024年7月施行・標準計算 ────────────────────────────────────────────────

/// 売買価格 1,000,000円（2024年8月・標準取引）
/// 期待: 税抜 50,000円 / 税額 5,000円 / 税込 55,000円
#[test]
fn mlitt_2024_price_1m() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(1_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 50_000);
    assert_eq!(result.tax_amount.as_yen(), 5_000);
    assert_eq!(result.total_with_tax.as_yen(), 55_000);
}

/// 売買価格 2,000,000円（tier1 上限境界）
/// 期待: 税抜 100,000円 / 税額 10,000円 / 税込 110,000円
#[test]
fn mlitt_2024_price_2m_boundary() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(2_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 100_000);
    assert_eq!(result.tax_amount.as_yen(), 10_000);
    assert_eq!(result.total_with_tax.as_yen(), 110_000);
}

/// 売買価格 3,000,000円（tier2）
/// 期待: 税抜 140,000円 / 税額 14,000円 / 税込 154,000円
#[test]
fn mlitt_2024_price_3m() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(3_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 140_000);
    assert_eq!(result.tax_amount.as_yen(), 14_000);
    assert_eq!(result.total_with_tax.as_yen(), 154_000);
}

/// 売買価格 4,000,000円（tier2 上限境界）
/// 期待: 税抜 180,000円 / 税額 18,000円 / 税込 198,000円
#[test]
fn mlitt_2024_price_4m_boundary() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(4_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    assert_eq!(result.tax_amount.as_yen(), 18_000);
    assert_eq!(result.total_with_tax.as_yen(), 198_000);
}

/// 売買価格 5,000,000円（tier1+2+3 全ティア適用）
/// 期待: 税抜 210,000円 / 税額 21,000円 / 税込 231,000円 / breakdown 3件
#[test]
fn mlitt_2024_price_5m_three_tiers() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(5_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 210_000);
    assert_eq!(result.tax_amount.as_yen(), 21_000);
    assert_eq!(result.total_with_tax.as_yen(), 231_000);
    assert_eq!(result.breakdown.len(), 3);
}

/// 売買価格 10,000,000円
/// 期待: 税抜 360,000円 / 税額 36,000円 / 税込 396,000円
#[test]
fn mlitt_2024_price_10m() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(10_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 360_000);
    assert_eq!(result.tax_amount.as_yen(), 36_000);
    assert_eq!(result.total_with_tax.as_yen(), 396_000);
}

/// 売買価格 30,000,000円
/// 期待: 税抜 960,000円 / 税額 96,000円 / 税込 1,056,000円
#[test]
fn mlitt_2024_price_30m() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(30_000_000, (2024, 8, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 960_000);
    assert_eq!(result.tax_amount.as_yen(), 96_000);
    assert_eq!(result.total_with_tax.as_yen(), 1_056_000);
}

// ─── 2024年7月施行・低廉な空き家特例 ────────────────────────────────────────

/// 売買価格 8,000,000円（特例適用・上限境界）
/// 通常計算 税抜 240,000円 → 上限 330,000円 にキャップ
/// 期待: 税抜 330,000円 / 税額 33,000円 / 税込 363,000円
#[test]
fn mlitt_2024_low_cost_special_8m() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_with_flag(
            8_000_000,
            (2024, 8, 1),
            RealEstateFlag::IsLowCostVacantHouse,
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 330_000);
    assert_eq!(result.tax_amount.as_yen(), 33_000);
    assert_eq!(result.total_with_tax.as_yen(), 363_000);
    assert!(result.low_cost_special_applied);
}

/// 売買価格 8,000,001円（特例対象外・上限を1円超過）
/// フラグがあっても価格が超過しているため特例は適用されない
#[test]
fn mlitt_2024_low_cost_special_not_applied_over_ceiling() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_with_flag(
            8_000_001,
            (2024, 8, 1),
            RealEstateFlag::IsLowCostVacantHouse,
        ),
        &params,
    )
    .unwrap();
    assert!(!result.low_cost_special_applied);
}

/// 売買価格 8,000,000円・フラグなし（特例は適用されない）
#[test]
fn mlitt_2024_low_cost_no_flag_no_special() {
    let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(8_000_000, (2024, 8, 1)), &params).unwrap();
    assert!(!result.low_cost_special_applied);
    assert_ne!(result.total_without_tax.as_yen(), 330_000);
}

// ─── 2019年10月施行・旧告示（特例なし）────────────────────────────────────────

/// 売買価格 5,000,000円（2019年12月・旧告示）
/// 特例なし・ティア計算は同じ
/// 期待: 税抜 210,000円 / 税額 21,000円 / 税込 231,000円
#[test]
fn mlitt_2019_price_5m() {
    let params = load_brokerage_fee_params((2019, 12, 1)).unwrap();
    let result = calculate_brokerage_fee(&ctx(5_000_000, (2019, 12, 1)), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 210_000);
    assert_eq!(result.tax_amount.as_yen(), 21_000);
    assert_eq!(result.total_with_tax.as_yen(), 231_000);
    assert!(params.low_cost_special.is_none());
}
