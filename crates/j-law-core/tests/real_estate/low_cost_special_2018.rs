//! 低廉特例テスト（2018年1月1日〜2024年6月30日）
//!
//! 平成29年国土交通省告示第98号（2017年12月8日公布・2018年1月1日施行）で導入された
//! 400万円以下の低廉な空き家等に関する報酬特例の検証。
//! この期間の特例は**売主のみ**に適用される。
//!
//! 参照:
//! - 国土交通省「宅地建物取引業法の解釈・運用の考え方」第46条関係
//! - https://www.mlit.go.jp/totikensangyo/const/1_6_bt_000083.html

#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_core::LegalDate;
use j_law_registry::load_brokerage_fee_params;

/// フラグなしのコンテキストを生成する。
fn ctx(price: u64, date: LegalDate) -> RealEstateContext {
    RealEstateContext {
        price,
        target_date: date,
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    }
}

/// 指定フラグを付けたコンテキストを生成する。
fn ctx_flags(price: u64, date: LegalDate, flags: &[RealEstateFlag]) -> RealEstateContext {
    let flags: HashSet<RealEstateFlag> = flags.iter().copied().collect();
    RealEstateContext {
        price,
        target_date: date,
        flags,
        policy: Box::new(StandardMliitPolicy),
    }
}

// ─── 2019年10月〜2024年6月（消費税10%・売主限定特例）──────────────────────────

/// 売買価格 3,000,000円・売主フラグあり・低廉特例フラグあり（2022年1月）
///
/// 通常計算:
///   tier1: 2,000,000 × 5% = 100,000
///   tier2: 1,000,000 × 4% = 40,000
///   合計(税抜): 140,000 円
///
/// 特例適用（売主・140,000 < 180,000）→ 180,000 円に引き上げ
///
/// 期待: 税抜 180,000円 / 税額 18,000円 / 税込 198,000円
#[test]
fn low_cost_2022_seller_3m_special_applied() {
    let date = LegalDate::new(2022, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(
            3_000_000,
            date,
            &[
                RealEstateFlag::IsLowCostVacantHouse,
                RealEstateFlag::IsSeller,
            ],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    assert_eq!(result.tax_amount.as_yen(), 18_000);
    assert_eq!(result.total_with_tax.as_yen(), 198_000);
    assert!(result.low_cost_special_applied);
}

/// 売買価格 3,000,000円・売主フラグなし（買主側）・低廉特例フラグあり（2022年1月）
///
/// 2018〜2024年の特例は売主のみ。買主側には適用されない。
/// 通常計算: 140,000 円
///
/// 期待: 税抜 140,000円 / 税額 14,000円 / 税込 154,000円
#[test]
fn low_cost_2022_buyer_3m_special_not_applied() {
    let date = LegalDate::new(2022, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(3_000_000, date, &[RealEstateFlag::IsLowCostVacantHouse]),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 140_000);
    assert_eq!(result.tax_amount.as_yen(), 14_000);
    assert_eq!(result.total_with_tax.as_yen(), 154_000);
    assert!(!result.low_cost_special_applied);
}

/// 売買価格 4,000,000円・売主フラグあり・低廉特例フラグあり（2022年1月）
///
/// 通常計算:
///   tier1: 2,000,000 × 5% = 100,000
///   tier2: 2,000,000 × 4% = 80,000
///   合計(税抜): 180,000 円
///
/// 通常計算 = 特例額 → 特例適用扱いだが結果は同じ
///
/// 期待: 税抜 180,000円 / 税額 18,000円 / 税込 198,000円
#[test]
fn low_cost_2022_seller_4m_boundary_equals_special() {
    let date = LegalDate::new(2022, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(
            4_000_000,
            date,
            &[
                RealEstateFlag::IsLowCostVacantHouse,
                RealEstateFlag::IsSeller,
            ],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    assert_eq!(result.tax_amount.as_yen(), 18_000);
    assert_eq!(result.total_with_tax.as_yen(), 198_000);
    assert!(result.low_cost_special_applied);
}

/// 売買価格 4,000,001円・売主フラグあり・低廉特例フラグあり（2022年1月）
///
/// 400万円を1円超過するため特例対象外（price_ceiling_inclusive = 4,000,000）。
/// 通常の3段階計算:
///   tier1: 100,000 / tier2: 80,000 / tier3: 1 × 3/100 = 0 (Floor)
///
/// 期待: 税抜 180,000円 / 特例不適用
#[test]
fn low_cost_2022_seller_4m_plus_1_special_not_applied() {
    let date = LegalDate::new(2022, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(
            4_000_001,
            date,
            &[
                RealEstateFlag::IsLowCostVacantHouse,
                RealEstateFlag::IsSeller,
            ],
        ),
        &params,
    )
    .unwrap();
    assert!(!result.low_cost_special_applied);
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
}

/// 売買価格 3,000,000円・フラグなし（2022年1月）
///
/// IsLowCostVacantHouse フラグなし → 特例不適用。
/// 通常計算: 140,000 円
#[test]
fn low_cost_2022_no_flag_no_special() {
    let date = LegalDate::new(2022, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(&ctx(3_000_000, date), &params).unwrap();
    assert!(!result.low_cost_special_applied);
    assert_eq!(result.total_without_tax.as_yen(), 140_000);
}

// ─── 2018年1月〜2019年9月（消費税8%・売主限定特例）──────────────────────────

/// 売買価格 3,000,000円・売主フラグあり・低廉特例フラグあり（2018年6月・消費税8%）
///
/// 通常計算: 140,000 円（税抜）
/// 特例適用（売主・140,000 < 180,000）→ 180,000 円に引き上げ
/// 消費税8%: 180,000 × 8% = 14,400 → Floor → 14,400 円
///
/// 期待: 税抜 180,000円 / 税額 14,400円 / 税込 194,400円
#[test]
fn low_cost_2018_seller_3m_special_applied() {
    let date = LegalDate::new(2018, 6, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(
            3_000_000,
            date,
            &[
                RealEstateFlag::IsLowCostVacantHouse,
                RealEstateFlag::IsSeller,
            ],
        ),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 180_000);
    // 180,000 × 8/100 = 14,400
    assert_eq!(result.tax_amount.as_yen(), 14_400);
    assert_eq!(result.total_with_tax.as_yen(), 194_400);
    assert!(result.low_cost_special_applied);
}

/// 売買価格 3,000,000円・買主側（2018年6月・消費税8%）
///
/// 2018年の特例は売主のみ。買主側には適用されない。
/// 通常計算: 税抜 140,000 / 税額 11,200 / 税込 151,200
#[test]
fn low_cost_2018_buyer_3m_special_not_applied() {
    let date = LegalDate::new(2018, 6, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(3_000_000, date, &[RealEstateFlag::IsLowCostVacantHouse]),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 140_000);
    // 140,000 × 8/100 = 11,200
    assert_eq!(result.tax_amount.as_yen(), 11_200);
    assert_eq!(result.total_with_tax.as_yen(), 151_200);
    assert!(!result.low_cost_special_applied);
}

// ─── 2024年7月以降（売主・買主双方に適用）────────────────────────────────────

/// 売買価格 5,000,000円・IsSeller フラグなし（買主側）・低廉特例フラグあり（2024年8月）
///
/// 2024年7月以降は seller_only = false なので、
/// IsSeller フラグがなくても（買主側でも）特例が適用される。
/// 500万円 ≤ 800万円なので特例対象内。
/// 通常計算: tier1=100,000 / tier2=80,000 / tier3=30,000 = 210,000
/// 210,000 < 330,000 → 330,000 に引き上げ
///
/// 期待: 税抜 330,000円 / 特例適用
#[test]
fn low_cost_2024_buyer_5m_special_applied_without_seller_flag() {
    let date = LegalDate::new(2024, 8, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(5_000_000, date, &[RealEstateFlag::IsLowCostVacantHouse]),
        &params,
    )
    .unwrap();
    // seller_only = false なので IsSeller なしでも特例適用
    assert!(result.low_cost_special_applied);
    assert_eq!(result.total_without_tax.as_yen(), 330_000);
}

/// 売買価格 8,000,001円・買主側・低廉特例フラグあり（2024年8月）
///
/// 800万円を1円超過するため特例対象外。
/// 通常計算: tier1=100,000 / tier2=80,000 / tier3=4,000,001×3%=120,000 = 300,000...
/// 正確: tier3 base = 8,000,001 - 4,000,000 = 4,000,001 × 3/100 = 120,000 (Floor)
/// 合計: 100,000 + 80,000 + 120,000 = 300,000
///
/// 期待: 税抜 300,000円 / 特例不適用
#[test]
fn low_cost_2024_buyer_8m_plus_1_special_not_applied() {
    let date = LegalDate::new(2024, 8, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(8_000_001, date, &[RealEstateFlag::IsLowCostVacantHouse]),
        &params,
    )
    .unwrap();
    assert!(!result.low_cost_special_applied);
    assert_eq!(result.total_without_tax.as_yen(), 300_000);
}

/// 売買価格 6,000,000円・買主フラグなし・低廉特例フラグあり（2024年8月）
///
/// 2024年7月以降は seller_only = false。IsSeller フラグなしでも適用対象。
/// 600万円は 800万円以下なので特例対象。
/// 通常計算: tier1=100,000 / tier2=80,000 / tier3=2,000,000×3%=60,000 = 240,000
/// 240,000 < 330,000 → 330,000 に引き上げ
///
/// 期待: 税抜 330,000円 / 特例適用
#[test]
fn low_cost_2024_buyer_6m_special_applied_no_seller_flag() {
    let date = LegalDate::new(2024, 8, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    let result = calculate_brokerage_fee(
        &ctx_flags(6_000_000, date, &[RealEstateFlag::IsLowCostVacantHouse]),
        &params,
    )
    .unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 330_000);
    assert!(result.low_cost_special_applied);
}
