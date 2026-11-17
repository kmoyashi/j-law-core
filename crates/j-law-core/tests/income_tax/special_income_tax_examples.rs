//! 付加税（復興特別所得税・防衛特別所得税）の年分別計算（Registry 連携）
//!
//! - 2013〜2026年分: 復興特別所得税 2.1%
//! - 2027〜2047年分: 復興特別所得税 1.1% + 防衛特別所得税 1%（令和8年法律第12号）
//! - 2048年分以後: 防衛特別所得税 1%
//!
//! 出典: 財務省「令和8年度税制改正の大綱」、財務省「令和8年度税制改正の解説（防衛特別所得税の創設等）」、
//! 国税庁「防衛特別所得税及び復興特別所得税（源泉徴収関係）Q&A」（令和8年5月）

#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculate_income_tax, IncomeTaxContext, IncomeTaxFlag, IncomeTaxResult, StandardIncomeTaxPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_income_tax_params;

fn income_tax(taxable_income: u64, date: LegalDate, apply_special_taxes: bool) -> IncomeTaxResult {
    let mut flags = HashSet::new();
    if apply_special_taxes {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }
    let ctx = IncomeTaxContext {
        taxable_income,
        target_date: date,
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };
    let params = load_income_tax_params(date).unwrap();
    calculate_income_tax(&ctx, &params).unwrap()
}

#[test]
fn year_2026_reconstruction_only() {
    let r = income_tax(1_000_000, LegalDate::new(2026, 12, 31), true);
    assert_eq!(r.base_tax.as_yen(), 50_000);
    assert_eq!(r.reconstruction_tax.as_yen(), 1_050);
    assert_eq!(r.defense_tax.as_yen(), 0);
    assert_eq!(r.total_tax.as_yen(), 51_000);
    assert!(r.reconstruction_tax_applied);
    assert!(!r.defense_tax_applied);
}

#[test]
fn years_2027_to_2047_reconstruction_and_defense() {
    for date in [
        LegalDate::new(2027, 1, 1),
        LegalDate::new(2038, 1, 1),
        LegalDate::new(2047, 12, 31),
    ] {
        let r = income_tax(1_000_000, date, true);
        assert_eq!(r.base_tax.as_yen(), 50_000);
        assert_eq!(r.reconstruction_tax.as_yen(), 550);
        assert_eq!(r.defense_tax.as_yen(), 500);
        // 合計税率 2.1% は改正前と同じため、申告納税額も変わらない。
        assert_eq!(r.total_tax.as_yen(), 51_000);
        assert!(r.reconstruction_tax_applied);
        assert!(r.defense_tax_applied);
    }
}

#[test]
fn from_2048_defense_only() {
    let r = income_tax(1_000_000, LegalDate::new(2048, 1, 1), true);
    assert_eq!(r.reconstruction_tax.as_yen(), 0);
    assert_eq!(r.defense_tax.as_yen(), 500);
    assert_eq!(r.total_tax.as_yen(), 50_500);
    assert!(!r.reconstruction_tax_applied);
    assert!(r.defense_tax_applied);
}

#[test]
fn special_taxes_are_skipped_without_flag() {
    let r = income_tax(1_000_000, LegalDate::new(2027, 1, 1), false);
    assert_eq!(r.reconstruction_tax.as_yen(), 0);
    assert_eq!(r.defense_tax.as_yen(), 0);
    assert_eq!(r.total_tax.as_yen(), 50_000);
}

#[test]
fn total_tax_is_unchanged_by_the_2027_reform() {
    // 付加税は合計額で端数処理する（防衛財源確保法 第5条の22第2項）ため、
    // 2026年分と2027年分で同じ課税所得なら申告納税額は一致する。
    for taxable_income in (1_000..=50_000_000).step_by(123_457) {
        let before = income_tax(taxable_income, LegalDate::new(2026, 1, 1), true);
        let after = income_tax(taxable_income, LegalDate::new(2027, 1, 1), true);
        assert_eq!(before.base_tax, after.base_tax);
        assert_eq!(
            before.total_tax, after.total_tax,
            "課税所得 {taxable_income}円"
        );
        assert_eq!(
            after.base_tax.as_yen()
                + after.reconstruction_tax.as_yen()
                + after.defense_tax.as_yen(),
            before.base_tax.as_yen() + before.reconstruction_tax.as_yen(),
            "課税所得 {taxable_income}円"
        );
    }
}
