package jlawcore_test

import (
	"strings"
	"testing"

	jlawcore "github.com/j-law-core/j-law-go"
)

// ─── 正常系 ──────────────────────────────────────────────────────────────────

// TestCalcBrokerageFee_5M は国土交通省告示の計算例（500万円）を検証する。
//
// 計算根拠:
//   - tier1: 2,000,000 × 5/100 = 100,000
//   - tier2: 2,000,000 × 4/100 =  80,000
//   - tier3: 1,000,000 × 3/100 =  30,000
//   - 税抜合計: 210,000
//   - 消費税: 21,000
//   - 税込合計: 231,000
func TestCalcBrokerageFee_5M(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.TotalWithoutTax != 210_000 {
		t.Errorf("TotalWithoutTax: got %d, want 210000", result.TotalWithoutTax)
	}
	if result.TotalWithTax != 231_000 {
		t.Errorf("TotalWithTax: got %d, want 231000", result.TotalWithTax)
	}
	if result.TaxAmount != 21_000 {
		t.Errorf("TaxAmount: got %d, want 21000", result.TaxAmount)
	}
	if result.LowCostSpecialApplied {
		t.Error("LowCostSpecialApplied: got true, want false")
	}
	if len(result.Breakdown) != 3 {
		t.Fatalf("Breakdown length: got %d, want 3", len(result.Breakdown))
	}
	if result.Breakdown[0].Result != 100_000 {
		t.Errorf("tier1 result: got %d, want 100000", result.Breakdown[0].Result)
	}
	if result.Breakdown[1].Result != 80_000 {
		t.Errorf("tier2 result: got %d, want 80000", result.Breakdown[1].Result)
	}
	if result.Breakdown[2].Result != 30_000 {
		t.Errorf("tier3 result: got %d, want 30000", result.Breakdown[2].Result)
	}
}

// TestCalcBrokerageFee_LowCostSpecial_8M は低廉な空き家特例（800万円）を検証する。
//
// 通常計算:
//   - tier1: 2,000,000 × 5/100 = 100,000
//   - tier2: 2,000,000 × 4/100 =  80,000
//   - tier3: 4,000,000 × 3/100 = 120,000  ← tier3 の対象は (8M - 4M) = 4M 円
//   - 税抜合計: 300,000
//
// 特例適用 (flag=true, 2024-07-01~):
//   - max(300,000, 330,000) = 330,000（保証額に引き上げ）
//   - 消費税: 33,000
//   - 税込合計: 363,000
func TestCalcBrokerageFee_LowCostSpecial_8M(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(8_000_000, 2024, 8, 1, true)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.TotalWithoutTax != 330_000 {
		t.Errorf("TotalWithoutTax: got %d, want 330000", result.TotalWithoutTax)
	}
	if result.TotalWithTax != 363_000 {
		t.Errorf("TotalWithTax: got %d, want 363000", result.TotalWithTax)
	}
	if result.TaxAmount != 33_000 {
		t.Errorf("TaxAmount: got %d, want 33000", result.TaxAmount)
	}
	if !result.LowCostSpecialApplied {
		t.Error("LowCostSpecialApplied: got false, want true")
	}
}

// TestCalcBrokerageFee_LowCostSpecialNotApplied は特例フラグなしの場合に通常計算になることを検証する。
//
// 計算根拠（8,000,000円、フラグなし）:
//   - tier1: 2,000,000 × 5/100 = 100,000
//   - tier2: 2,000,000 × 4/100 =  80,000
//   - tier3: 4,000,000 × 3/100 = 120,000
//   - 税抜合計: 300,000
//   - 消費税: 30,000
//   - 税込合計: 330,000
func TestCalcBrokerageFee_LowCostSpecialNotApplied(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(8_000_000, 2024, 8, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.TotalWithoutTax != 300_000 {
		t.Errorf("TotalWithoutTax: got %d, want 300000", result.TotalWithoutTax)
	}
	if result.TotalWithTax != 330_000 {
		t.Errorf("TotalWithTax: got %d, want 330000", result.TotalWithTax)
	}
	if result.TaxAmount != 30_000 {
		t.Errorf("TaxAmount: got %d, want 30000", result.TaxAmount)
	}
	if result.LowCostSpecialApplied {
		t.Error("LowCostSpecialApplied: got true, want false")
	}
}

// TestCalcBrokerageFee_2019 は 2024-07-01 施行前のパラメータを検証する（特例なし）。
func TestCalcBrokerageFee_2019(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2019, 12, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.TotalWithoutTax != 210_000 {
		t.Errorf("TotalWithoutTax: got %d, want 210000", result.TotalWithoutTax)
	}
	if result.LowCostSpecialApplied {
		t.Error("LowCostSpecialApplied: should be false for 2019 params")
	}
}

// TestCalcBrokerageFee_Breakdown は内訳の基本フィールドを検証する。
func TestCalcBrokerageFee_Breakdown(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	for _, step := range result.Breakdown {
		if step.Label == "" {
			t.Error("BreakdownStep.Label must not be empty")
		}
		if step.RateDenom == 0 {
			t.Error("BreakdownStep.RateDenom must not be zero")
		}
	}
}

// ─── 異常系 ──────────────────────────────────────────────────────────────────

// TestCalcBrokerageFee_DateOutOfRange は対象日が範囲外の場合にエラーを返すことを検証する。
func TestCalcBrokerageFee_DateOutOfRange(t *testing.T) {
	_, err := jlawcore.CalcBrokerageFee(5_000_000, 2019, 9, 30, false)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
	if !strings.Contains(err.Error(), "2019-09-30") {
		t.Errorf("error message should contain the invalid date, got: %s", err.Error())
	}
}
