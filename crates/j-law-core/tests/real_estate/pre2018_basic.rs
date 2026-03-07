//! 2018年以前（低廉特例導入前）の媒介報酬計算テスト
//!
//! 平成29年国土交通省告示第98号（2018年1月1日施行）で導入された低廉特例以前においても、
//! 宅地建物取引業法第46条に基づく媒介報酬上限は存在する。
//! 消費税は消費税ドメインから自動取得される。
//!
//! 参照:
//! - 宅地建物取引業法 第46条第1項
//! - 昭和45年10月23日建設省告示第1552号（1970年12月1日施行・3段階ティア制確立）

#![allow(clippy::disallowed_methods)] // テストコードでは unwrap() 使用可能

use std::collections::HashSet;

use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_core::LegalDate;
use j_law_registry::load_brokerage_fee_params;

fn ctx(price: u64, date: LegalDate) -> RealEstateContext {
    RealEstateContext {
        price,
        target_date: date,
        flags: HashSet::new(),
        policy: Box::new(StandardMliitPolicy),
    }
}

fn ctx_flags(price: u64, date: LegalDate, flags: &[RealEstateFlag]) -> RealEstateContext {
    let flags: HashSet<RealEstateFlag> = flags.iter().copied().collect();
    RealEstateContext {
        price,
        target_date: date,
        flags,
        policy: Box::new(StandardMliitPolicy),
    }
}

// ─── 2017年12月31日（特例導入前・消費税8%）───────────────────────────────────

/// 売買価格 5,000,000円（2017年12月31日・特例導入前）
///
/// ティア計算:
///   tier1: 2,000,000 × 5% = 100,000
///   tier2: 2,000,000 × 4% = 80,000
///   tier3: 1,000,000 × 3% = 30,000
///   合計(税抜): 210,000 円
///
/// 消費税8%（消費税ドメインから取得）:
///   210,000 × 8/100 = 16,800 円
///
/// 期待: 税抜 210,000円 / 税額 16,800円 / 税込 226,800円
#[test]
fn pre2018_2017_price_5m_no_special() {
    let date = LegalDate::new(2017, 12, 31);
    let params = load_brokerage_fee_params(date).unwrap();
    // 低廉特例は存在しない
    assert!(params.low_cost_special.is_none());
    let result = calculate_brokerage_fee(&ctx(5_000_000, date), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 210_000);
    assert_eq!(result.tax_amount.as_yen(), 16_800);
    assert_eq!(result.total_with_tax.as_yen(), 226_800);
    assert!(!result.low_cost_special_applied);
}

/// 売買価格 3,000,000円・IsLowCostVacantHouse フラグあり（2017年12月31日）
///
/// 低廉特例は 2018年1月1日以降にしか存在しない。
/// フラグがあっても特例は適用されない。
///
/// ティア計算:
///   tier1: 2,000,000 × 5% = 100,000
///   tier2: 1,000,000 × 4% = 40,000
///   合計(税抜): 140,000 円
///
/// 期待: 税抜 140,000円 / 特例不適用
#[test]
fn pre2018_low_cost_flag_no_special() {
    let date = LegalDate::new(2017, 12, 31);
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
    assert!(!result.low_cost_special_applied);
    assert_eq!(result.total_without_tax.as_yen(), 140_000);
}

/// 2018年1月1日と2017年12月31日の境界テスト（低廉特例の有効化）
///
/// 2017-12-31: 低廉特例なし
/// 2018-01-01: 低廉特例あり（売主限定・400万円以下）
#[test]
fn boundary_2018_01_01_special_activates() {
    let date_before = LegalDate::new(2017, 12, 31);
    let date_after = LegalDate::new(2018, 1, 1);

    let params_before = load_brokerage_fee_params(date_before).unwrap();
    let params_after = load_brokerage_fee_params(date_after).unwrap();

    assert!(params_before.low_cost_special.is_none());
    assert!(params_after.low_cost_special.is_some());

    let special_after = params_after.low_cost_special.as_ref().unwrap();
    assert_eq!(special_after.price_ceiling_inclusive, 4_000_000);
    assert_eq!(special_after.fee_ceiling_exclusive_tax, 180_000);
    assert!(special_after.seller_only);
}

/// 売買価格 1,000,000円（1990年・消費税3%時代）
///
/// ティア計算:
///   tier1: 1,000,000 × 5% = 50,000
///   合計(税抜): 50,000 円
///
/// 消費税3%（消費税ドメインから取得）:
///   50,000 × 3/100 = 1,500 円
///
/// 期待: 税抜 50,000円 / 税額 1,500円 / 税込 51,500円
#[test]
fn pre2018_1990_price_1m_3pct_tax() {
    let date = LegalDate::new(1990, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    assert!(params.low_cost_special.is_none());
    // 1990年は消費税3%
    assert_eq!(params.consumption_tax.standard_rate.numer, 3);
    let result = calculate_brokerage_fee(&ctx(1_000_000, date), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 50_000);
    assert_eq!(result.tax_amount.as_yen(), 1_500);
    assert_eq!(result.total_with_tax.as_yen(), 51_500);
}

/// 売買価格 5,000,000円（2000年・消費税5%時代）
///
/// ティア計算:
///   tier1: 2,000,000 × 5% = 100,000
///   tier2: 2,000,000 × 4% = 80,000
///   tier3: 1,000,000 × 3% = 30,000
///   合計(税抜): 210,000 円
///
/// 消費税5%（消費税ドメインから取得）:
///   210,000 × 5/100 = 10,500 円
///
/// 期待: 税抜 210,000円 / 税額 10,500円 / 税込 220,500円
#[test]
fn pre2018_2000_price_5m_5pct_tax() {
    let date = LegalDate::new(2000, 1, 1);
    let params = load_brokerage_fee_params(date).unwrap();
    assert!(params.low_cost_special.is_none());
    // 2000年は消費税5%
    assert_eq!(params.consumption_tax.standard_rate.numer, 5);
    let result = calculate_brokerage_fee(&ctx(5_000_000, date), &params).unwrap();
    assert_eq!(result.total_without_tax.as_yen(), 210_000);
    assert_eq!(result.tax_amount.as_yen(), 10_500);
    assert_eq!(result.total_with_tax.as_yen(), 220_500);
}
