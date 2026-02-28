//! 所得税計算例テスト
//!
//! 出典: 国税庁「所得税の税率」
//! https://www.nta.go.jp/taxes/shiraberu/taxanswer/shotoku/2260.htm
//!
//! 令和6年分の所得税速算表（所得税法 第89条第1項）に基づく。
//! 復興特別所得税（復興財源確保法 第13条）を含む。

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    params::{IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams},
    policy::StandardIncomeTaxPolicy,
};

/// 令和6年分の所得税速算表（所得税法 第89条第1項）。
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

fn ctx_with_reconstruction(income: u64) -> IncomeTaxContext {
    let mut flags = HashSet::new();
    flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    IncomeTaxContext {
        taxable_income: income,
        target_date: (2024, 1, 1),
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    }
}

fn ctx_without_reconstruction(income: u64) -> IncomeTaxContext {
    IncomeTaxContext {
        taxable_income: income,
        target_date: (2024, 1, 1),
        flags: HashSet::new(),
        policy: Box::new(StandardIncomeTaxPolicy),
    }
}

// ─── 課税所得金額 0円 ────────────────────────────────────────────────────────

#[test]
fn zero_income() {
    let result = calculate_income_tax(&ctx_without_reconstruction(0), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 0);
    assert_eq!(result.reconstruction_tax.as_yen(), 0);
    assert_eq!(result.total_tax.as_yen(), 0);
    assert!(result.breakdown.is_empty());
}

// ─── ブラケット1: 195万円以下 (5%, 控除0) ────────────────────────────────────

/// 課税所得 1,950,000円（ブラケット1上限）
/// 基準税額: 1,950,000 × 5% - 0 = 97,500
/// 復興税: 97,500 × 21/1000 = 2,047
/// 合計: 97,500 + 2,047 = 99,547 → 99,500
#[test]
fn bracket1_1_950_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(1_950_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 97_500);
    assert_eq!(result.reconstruction_tax.as_yen(), 2_047);
    assert_eq!(result.total_tax.as_yen(), 99_500);
    assert!(result.reconstruction_tax_applied);
}

/// 課税所得 1,000,000円
/// 基準税額: 1,000,000 × 5% - 0 = 50,000
/// 復興税: 50,000 × 21/1000 = 1,050
/// 合計: 50,000 + 1,050 = 51,050 → 51,000
#[test]
fn bracket1_1_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(1_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 50_000);
    assert_eq!(result.reconstruction_tax.as_yen(), 1_050);
    assert_eq!(result.total_tax.as_yen(), 51_000);
}

// ─── ブラケット2: 195万円超330万円以下 (10%, 控除97,500) ─────────────────────

/// 課税所得 3,000,000円
/// 基準税額: 3,000,000 × 10% - 97,500 = 202,500
/// 復興税: 202,500 × 21/1000 = 4,252
/// 合計: 202,500 + 4,252 = 206,752 → 206,700
#[test]
fn bracket2_3_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(3_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 202_500);
    assert_eq!(result.reconstruction_tax.as_yen(), 4_252);
    assert_eq!(result.total_tax.as_yen(), 206_700);
}

// ─── ブラケット3: 330万円超695万円以下 (20%, 控除427,500) ────────────────────

/// 課税所得 5,000,000円
/// 基準税額: 5,000,000 × 20% - 427,500 = 572,500
/// 復興税: 572,500 × 21/1000 = 12,022
/// 合計: 572,500 + 12,022 = 584,522 → 584,500
#[test]
fn bracket3_5_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(5_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 572_500);
    assert_eq!(result.reconstruction_tax.as_yen(), 12_022);
    assert_eq!(result.total_tax.as_yen(), 584_500);
    assert_eq!(result.breakdown.len(), 1);
    assert_eq!(result.breakdown[0].deduction, 427_500);
}

// ─── ブラケット4: 695万円超900万円以下 (23%, 控除636,000) ────────────────────

/// 課税所得 8,000,000円
/// 基準税額: 8,000,000 × 23% - 636,000 = 1,204,000
/// 復興税: 1,204,000 × 21/1000 = 25,284
/// 合計: 1,204,000 + 25,284 = 1,229,284 → 1,229,200
#[test]
fn bracket4_8_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(8_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 1_204_000);
    assert_eq!(result.reconstruction_tax.as_yen(), 25_284);
    assert_eq!(result.total_tax.as_yen(), 1_229_200);
}

// ─── ブラケット5: 900万円超1800万円以下 (33%, 控除1,536,000) ─────────────────

/// 課税所得 10,000,000円
/// 基準税額: 10,000,000 × 33% - 1,536,000 = 1,764,000
/// 復興税: 1,764,000 × 21/1000 = 37,044
/// 合計: 1,764,000 + 37,044 = 1,801,044 → 1,801,000
#[test]
fn bracket5_10_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(10_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 1_764_000);
    assert_eq!(result.reconstruction_tax.as_yen(), 37_044);
    assert_eq!(result.total_tax.as_yen(), 1_801_000);
}

// ─── ブラケット6: 1800万円超4000万円以下 (40%, 控除2,796,000) ────────────────

/// 課税所得 20,000,000円
/// 基準税額: 20,000,000 × 40% - 2,796,000 = 5,204,000
/// 復興税: 5,204,000 × 21/1000 = 109,284
/// 合計: 5,204,000 + 109,284 = 5,313,284 → 5,313,200
#[test]
fn bracket6_20_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(20_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 5_204_000);
    assert_eq!(result.reconstruction_tax.as_yen(), 109_284);
    assert_eq!(result.total_tax.as_yen(), 5_313_200);
}

// ─── ブラケット7: 4000万円超 (45%, 控除4,796,000) ───────────────────────────

/// 課税所得 50,000,000円
/// 基準税額: 50,000,000 × 45% - 4,796,000 = 17,704,000
/// 復興税: 17,704,000 × 21/1000 = 371,784
/// 合計: 17,704,000 + 371,784 = 18,075,784 → 18,075,700
#[test]
fn bracket7_50_000_000() {
    let result =
        calculate_income_tax(&ctx_with_reconstruction(50_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 17_704_000);
    assert_eq!(result.reconstruction_tax.as_yen(), 371_784);
    assert_eq!(result.total_tax.as_yen(), 18_075_700);
}

// ─── 復興特別所得税なし ─────────────────────────────────────────────────────

/// 課税所得 5,000,000円（復興税フラグなし）
/// 基準税額: 572,500 / 復興税: 0 / 合計: 572,500
#[test]
fn without_reconstruction_tax() {
    let result =
        calculate_income_tax(&ctx_without_reconstruction(5_000_000), &tax_params_2024()).unwrap();
    assert_eq!(result.base_tax.as_yen(), 572_500);
    assert_eq!(result.reconstruction_tax.as_yen(), 0);
    assert_eq!(result.total_tax.as_yen(), 572_500);
    assert!(!result.reconstruction_tax_applied);
}

// ─── エラーケース ───────────────────────────────────────────────────────────

/// 復興税フラグありだがパラメータに復興税データがない → エラー
#[test]
fn missing_reconstruction_params_errors() {
    let params = IncomeTaxParams {
        brackets: vec![IncomeTaxBracket {
            label: "195万円以下".into(),
            income_from: 0,
            income_to_inclusive: Some(1_950_000),
            rate_numer: 5,
            rate_denom: 100,
            deduction: 0,
        }],
        reconstruction_tax: None,
    };
    let ctx = ctx_with_reconstruction(1_000_000);
    let result = calculate_income_tax(&ctx, &params);
    assert!(result.is_err());
}
