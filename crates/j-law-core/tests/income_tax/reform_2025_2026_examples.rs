//! 令和7年度・令和8年度税制改正による人的控除の改正（Registry 連携）
//!
//! 期待値は以下の国税庁公表資料の表に基づく。
//! - タックスアンサー No.1199 基礎控除
//! - タックスアンサー No.1177 特定親族特別控除
//! - タックスアンサー No.1191 配偶者控除
//! - 「源泉所得税の改正のあらまし」（令和8年4月）

#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::income_tax::{
    calculate_income_deductions, calculate_income_tax_assessment, DependentDeductionInput,
    ExpenseDeductionInput, IncomeDeductionContext, IncomeDeductionInput, IncomeDeductionKind,
    IncomeDeductionResult, IncomeTaxAssessmentContext, IncomeTaxFlag, PersonalDeductionInput,
    SpecificRelativeDeductionInput, SpouseDeductionInput, StandardIncomeTaxPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::{load_income_tax_deduction_params, load_income_tax_params};

fn context(
    date: LegalDate,
    total_income_amount: u64,
    personal: PersonalDeductionInput,
) -> IncomeDeductionContext {
    IncomeDeductionContext {
        total_income_amount,
        target_date: date,
        deductions: IncomeDeductionInput {
            personal,
            expense: ExpenseDeductionInput::default(),
        },
    }
}

fn deductions(
    date: LegalDate,
    total_income_amount: u64,
    personal: PersonalDeductionInput,
) -> IncomeDeductionResult {
    let params = load_income_tax_deduction_params(date).unwrap();
    calculate_income_deductions(&context(date, total_income_amount, personal), &params).unwrap()
}

fn amount_of(result: &IncomeDeductionResult, kind: IncomeDeductionKind) -> Option<u64> {
    result
        .breakdown
        .iter()
        .find(|line| line.kind == kind)
        .map(|line| line.amount.as_yen())
}

fn basic_deduction(date: LegalDate, total_income_amount: u64) -> u64 {
    let result = deductions(date, total_income_amount, PersonalDeductionInput::default());
    amount_of(&result, IncomeDeductionKind::Basic).unwrap()
}

fn assert_basic_deductions(date: LegalDate, cases: &[(u64, u64)]) {
    for &(income, expected) in cases {
        assert_eq!(
            basic_deduction(date, income),
            expected,
            "{} 合計所得金額 {income}円",
            date.to_date_str()
        );
    }
}

// ─── 基礎控除 ─────────────────────────────────────────────────────────────────

#[test]
fn basic_deduction_2024_is_unchanged() {
    assert_basic_deductions(
        LegalDate::new(2024, 12, 31),
        &[
            (0, 480_000),
            (24_000_000, 480_000),
            (24_000_001, 320_000),
            (25_000_001, 0),
        ],
    );
}

#[test]
fn basic_deduction_2025() {
    assert_basic_deductions(
        LegalDate::new(2025, 1, 1),
        &[
            (0, 950_000),
            (1_320_000, 950_000),
            (1_320_001, 880_000),
            (3_360_000, 880_000),
            (3_360_001, 680_000),
            (4_890_000, 680_000),
            (4_890_001, 630_000),
            (6_550_000, 630_000),
            (6_550_001, 580_000),
            (23_500_000, 580_000),
            (23_500_001, 480_000),
            (24_000_001, 320_000),
            (24_500_001, 160_000),
            (25_000_001, 0),
        ],
    );
}

#[test]
fn basic_deduction_2026_and_2027() {
    for date in [LegalDate::new(2026, 1, 1), LegalDate::new(2027, 12, 31)] {
        assert_basic_deductions(
            date,
            &[
                (0, 1_040_000),
                (1_320_001, 1_040_000),
                (4_890_000, 1_040_000),
                (4_890_001, 670_000),
                (6_550_000, 670_000),
                (6_550_001, 620_000),
                (23_500_000, 620_000),
                (23_500_001, 480_000),
                (25_000_001, 0),
            ],
        );
    }
}

#[test]
fn basic_deduction_2028_onward() {
    assert_basic_deductions(
        LegalDate::new(2028, 1, 1),
        &[
            (1_320_000, 990_000),
            (1_320_001, 620_000),
            (23_500_000, 620_000),
            (23_500_001, 480_000),
            (25_000_001, 0),
        ],
    );
}

#[test]
fn basic_deduction_switches_at_year_boundaries() {
    assert_eq!(
        basic_deduction(LegalDate::new(2024, 12, 31), 1_000_000),
        480_000
    );
    assert_eq!(
        basic_deduction(LegalDate::new(2025, 1, 1), 1_000_000),
        950_000
    );
    assert_eq!(
        basic_deduction(LegalDate::new(2025, 12, 31), 4_000_000),
        680_000
    );
    assert_eq!(
        basic_deduction(LegalDate::new(2026, 1, 1), 4_000_000),
        1_040_000
    );
    assert_eq!(
        basic_deduction(LegalDate::new(2027, 12, 31), 1_000_000),
        1_040_000
    );
    assert_eq!(
        basic_deduction(LegalDate::new(2028, 1, 1), 1_000_000),
        990_000
    );
}

// ─── 配偶者控除の所得要件 ──────────────────────────────────────────────────────

fn spouse_deduction(date: LegalDate, spouse_total_income_amount: u64) -> u64 {
    let result = deductions(
        date,
        5_000_000,
        PersonalDeductionInput {
            spouse: Some(SpouseDeductionInput {
                spouse_total_income_amount,
                is_same_household: true,
                is_elderly: false,
            }),
            ..PersonalDeductionInput::default()
        },
    );
    amount_of(&result, IncomeDeductionKind::Spouse).unwrap()
}

#[test]
fn spouse_income_requirement_by_year() {
    // 令和2〜6年分: 48万円以下
    assert_eq!(
        spouse_deduction(LegalDate::new(2024, 1, 1), 480_000),
        380_000
    );
    assert_eq!(spouse_deduction(LegalDate::new(2024, 1, 1), 480_001), 0);
    // 令和7年分: 58万円以下
    assert_eq!(
        spouse_deduction(LegalDate::new(2025, 1, 1), 580_000),
        380_000
    );
    assert_eq!(spouse_deduction(LegalDate::new(2025, 1, 1), 580_001), 0);
    // 令和8年分以後: 62万円以下
    assert_eq!(
        spouse_deduction(LegalDate::new(2026, 1, 1), 620_000),
        380_000
    );
    assert_eq!(spouse_deduction(LegalDate::new(2026, 1, 1), 620_001), 0);
    assert_eq!(
        spouse_deduction(LegalDate::new(2028, 1, 1), 620_000),
        380_000
    );
}

// ─── 特定親族特別控除 ──────────────────────────────────────────────────────────

fn with_specific_relatives(incomes: &[u64]) -> PersonalDeductionInput {
    PersonalDeductionInput {
        specific_relatives: incomes
            .iter()
            .map(|&total_income_amount| SpecificRelativeDeductionInput {
                total_income_amount,
            })
            .collect(),
        ..PersonalDeductionInput::default()
    }
}

fn specific_relative_deduction(date: LegalDate, relative_income: u64) -> Option<u64> {
    let result = deductions(date, 5_000_000, with_specific_relatives(&[relative_income]));
    amount_of(&result, IncomeDeductionKind::SpecificRelative)
}

#[test]
fn specific_relative_deduction_does_not_exist_before_2025() {
    assert_eq!(
        specific_relative_deduction(LegalDate::new(2024, 12, 31), 700_000),
        None
    );
}

#[test]
fn specific_relative_deduction_2025_table() {
    let date = LegalDate::new(2025, 1, 1);
    let cases = [
        (580_000, 0),
        (580_001, 630_000),
        (850_000, 630_000),
        (850_001, 610_000),
        (900_001, 510_000),
        (950_001, 410_000),
        (1_000_001, 310_000),
        (1_050_001, 210_000),
        (1_100_001, 110_000),
        (1_150_001, 60_000),
        (1_200_001, 30_000),
        (1_230_000, 30_000),
        (1_230_001, 0),
    ];
    for (income, expected) in cases {
        assert_eq!(
            specific_relative_deduction(date, income),
            Some(expected),
            "特定親族の合計所得金額 {income}円"
        );
    }
}

#[test]
fn specific_relative_lower_bound_moves_to_620k_in_2026() {
    // 令和7年分は58万円超から対象、令和8年分以後は62万円超から対象（62万円以下は扶養親族）。
    assert_eq!(
        specific_relative_deduction(LegalDate::new(2025, 12, 31), 600_000),
        Some(630_000)
    );
    assert_eq!(
        specific_relative_deduction(LegalDate::new(2026, 1, 1), 600_000),
        Some(0)
    );
    assert_eq!(
        specific_relative_deduction(LegalDate::new(2026, 1, 1), 620_001),
        Some(630_000)
    );
    assert_eq!(
        specific_relative_deduction(LegalDate::new(2028, 1, 1), 1_230_000),
        Some(30_000)
    );
}

#[test]
fn specific_relative_deduction_sums_multiple_relatives() {
    let result = deductions(
        LegalDate::new(2026, 1, 1),
        5_000_000,
        with_specific_relatives(&[700_000, 1_000_000, 1_500_000]),
    );
    assert_eq!(
        amount_of(&result, IncomeDeductionKind::SpecificRelative),
        Some(630_000 + 410_000)
    );
}

// ─── 通し計算 ─────────────────────────────────────────────────────────────────

#[test]
fn assessment_2026_reflects_raised_basic_deduction() {
    // 合計所得金額 300万円・社会保険料 45万円・扶養なし（令和8年分）
    // 所得控除 = 基礎控除 104万円 + 社会保険料控除 45万円 = 149万円
    // 課税所得 = 151万円 → 基準所得税額 = 151万円 × 5% = 75,500円
    // 復興特別所得税 = 75,500 × 2.1% = 1,585円（切捨て）
    // 申告納税額 = 77,085円 → 100円未満切捨て 77,000円
    let date = LegalDate::new(2026, 1, 1);
    let mut flags = HashSet::new();
    flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    let ctx = IncomeTaxAssessmentContext {
        deduction_context: IncomeDeductionContext {
            total_income_amount: 3_000_000,
            target_date: date,
            deductions: IncomeDeductionInput {
                personal: PersonalDeductionInput {
                    spouse: None,
                    dependent: DependentDeductionInput::default(),
                    specific_relatives: vec![],
                },
                expense: ExpenseDeductionInput {
                    social_insurance_premium_paid: 450_000,
                    ..ExpenseDeductionInput::default()
                },
            },
        },
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let deduction_params = load_income_tax_deduction_params(date).unwrap();
    let tax_params = load_income_tax_params(date).unwrap();
    let result = calculate_income_tax_assessment(&ctx, &deduction_params, &tax_params).unwrap();

    assert_eq!(result.deductions.total_deductions.as_yen(), 1_490_000);
    assert_eq!(result.deductions.taxable_income.as_yen(), 1_510_000);
    assert_eq!(result.tax.base_tax.as_yen(), 75_500);
    assert_eq!(result.tax.reconstruction_tax.as_yen(), 1_585);
    assert_eq!(result.tax.total_tax.as_yen(), 77_000);
}
