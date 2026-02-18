//! 境界値テスト
//!
//! 各ブラケット境界（195万/330万/695万/900万/1800万/4000万）での挙動を検証する。

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::IncomeTaxContext,
    params::{IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams},
    policy::StandardIncomeTaxPolicy,
};

fn tax_params_2024() -> IncomeTaxParams {
    IncomeTaxParams {
        brackets: vec![
            IncomeTaxBracket {
                label: "195万円以下".into(),
                income_from: 0,
                income_to_inclusive: Some(1_950_000),
                rate_numer: 5,
                rate_denom: 100,
                deduction: 0,
            },
            IncomeTaxBracket {
                label: "195万円超330万円以下".into(),
                income_from: 1_950_001,
                income_to_inclusive: Some(3_300_000),
                rate_numer: 10,
                rate_denom: 100,
                deduction: 97_500,
            },
            IncomeTaxBracket {
                label: "330万円超695万円以下".into(),
                income_from: 3_300_001,
                income_to_inclusive: Some(6_950_000),
                rate_numer: 20,
                rate_denom: 100,
                deduction: 427_500,
            },
            IncomeTaxBracket {
                label: "695万円超900万円以下".into(),
                income_from: 6_950_001,
                income_to_inclusive: Some(9_000_000),
                rate_numer: 23,
                rate_denom: 100,
                deduction: 636_000,
            },
            IncomeTaxBracket {
                label: "900万円超1800万円以下".into(),
                income_from: 9_000_001,
                income_to_inclusive: Some(18_000_000),
                rate_numer: 33,
                rate_denom: 100,
                deduction: 1_536_000,
            },
            IncomeTaxBracket {
                label: "1800万円超4000万円以下".into(),
                income_from: 18_000_001,
                income_to_inclusive: Some(40_000_000),
                rate_numer: 40,
                rate_denom: 100,
                deduction: 2_796_000,
            },
            IncomeTaxBracket {
                label: "4000万円超".into(),
                income_from: 40_000_001,
                income_to_inclusive: None,
                rate_numer: 45,
                rate_denom: 100,
                deduction: 4_796_000,
            },
        ],
        reconstruction_tax: Some(ReconstructionTaxParams {
            rate_numer: 21,
            rate_denom: 1000,
            effective_from_year: 2013,
            effective_to_year_inclusive: 2037,
        }),
    }
}

fn ctx(income: u64) -> IncomeTaxContext {
    IncomeTaxContext {
        taxable_income: income,
        target_date: (2024, 1, 1),
        flags: HashSet::new(),
        policy: Box::new(StandardIncomeTaxPolicy),
    }
}

// ─── 195万円境界 ─────────────────────────────────────────────────────────────

/// 1,950,000円（ブラケット1の上限ちょうど）
#[test]
fn boundary_bracket1_upper() {
    let result = calculate_income_tax(&ctx(1_950_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 97_500);
    assert_eq!(result.breakdown[0].label, "195万円以下");
}

/// 1,950,001円（ブラケット2の下限ちょうど）
/// 1,950,001 × 10% - 97,500 = 97,500
#[test]
fn boundary_bracket2_lower() {
    let result = calculate_income_tax(&ctx(1_950_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 97_500);
    assert_eq!(result.breakdown[0].label, "195万円超330万円以下");
}

// ─── 330万円境界 ─────────────────────────────────────────────────────────────

/// 3,300,000円（ブラケット2の上限ちょうど）
/// 3,300,000 × 10% - 97,500 = 232,500
#[test]
fn boundary_bracket2_upper() {
    let result = calculate_income_tax(&ctx(3_300_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 232_500);
    assert_eq!(result.breakdown[0].label, "195万円超330万円以下");
}

/// 3,300,001円（ブラケット3の下限ちょうど）
/// 3,300,001 × 20% - 427,500 = 232,500
#[test]
fn boundary_bracket3_lower() {
    let result = calculate_income_tax(&ctx(3_300_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 232_500);
    assert_eq!(result.breakdown[0].label, "330万円超695万円以下");
}

// ─── 695万円境界 ─────────────────────────────────────────────────────────────

/// 6,950,000円（ブラケット3の上限ちょうど）
/// 6,950,000 × 20% - 427,500 = 962,500
#[test]
fn boundary_bracket3_upper() {
    let result = calculate_income_tax(&ctx(6_950_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 962_500);
    assert_eq!(result.breakdown[0].label, "330万円超695万円以下");
}

/// 6,950,001円（ブラケット4の下限ちょうど）
/// 6,950,001 × 23% - 636,000 = 962,500
#[test]
fn boundary_bracket4_lower() {
    let result = calculate_income_tax(&ctx(6_950_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 962_500);
    assert_eq!(result.breakdown[0].label, "695万円超900万円以下");
}

// ─── 900万円境界 ─────────────────────────────────────────────────────────────

/// 9,000,000円（ブラケット4の上限ちょうど）
/// 9,000,000 × 23% - 636,000 = 1,434,000
#[test]
fn boundary_bracket4_upper() {
    let result = calculate_income_tax(&ctx(9_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 1_434_000);
    assert_eq!(result.breakdown[0].label, "695万円超900万円以下");
}

/// 9,000,001円（ブラケット5の下限ちょうど）
/// 9,000,001 × 33% - 1,536,000 = 1,434,000
#[test]
fn boundary_bracket5_lower() {
    let result = calculate_income_tax(&ctx(9_000_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 1_434_000);
    assert_eq!(result.breakdown[0].label, "900万円超1800万円以下");
}

// ─── 1800万円境界 ────────────────────────────────────────────────────────────

/// 18,000,000円（ブラケット5の上限ちょうど）
/// 18,000,000 × 33% - 1,536,000 = 4,404,000
#[test]
fn boundary_bracket5_upper() {
    let result = calculate_income_tax(&ctx(18_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 4_404_000);
    assert_eq!(result.breakdown[0].label, "900万円超1800万円以下");
}

/// 18,000,001円（ブラケット6の下限ちょうど）
/// 18,000,001 × 40% - 2,796,000 = 4,404,000
#[test]
fn boundary_bracket6_lower() {
    let result = calculate_income_tax(&ctx(18_000_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 4_404_000);
    assert_eq!(result.breakdown[0].label, "1800万円超4000万円以下");
}

// ─── 4000万円境界 ────────────────────────────────────────────────────────────

/// 40,000,000円（ブラケット6の上限ちょうど）
/// 40,000,000 × 40% - 2,796,000 = 13,204,000
#[test]
fn boundary_bracket6_upper() {
    let result = calculate_income_tax(&ctx(40_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 13_204_000);
    assert_eq!(result.breakdown[0].label, "1800万円超4000万円以下");
}

/// 40,000,001円（ブラケット7の下限ちょうど）
/// floor(40,000,001 × 45 / 100) - 4,796,000 = 18,000,000 - 4,796,000 = 13,204,000
#[test]
fn boundary_bracket7_lower() {
    let result = calculate_income_tax(&ctx(40_000_001), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 13_204_000);
    assert_eq!(result.breakdown[0].label, "4000万円超");
}
