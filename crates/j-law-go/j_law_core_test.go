package jlawcore_test

import (
	"encoding/json"
	"os"
	"strings"
	"testing"

	jlawcore "github.com/kmoyashi/j-law-go"
)

// ─── フィクスチャ型定義 ──────────────────────────────────────────────────────

type brokerageFeeInput struct {
	Price                uint64 `json:"price"`
	Year                 int    `json:"year"`
	Month                int    `json:"month"`
	Day                  int    `json:"day"`
	IsLowCostVacantHouse bool   `json:"is_low_cost_vacant_house"`
	IsSeller             bool   `json:"is_seller"`
}

type brokerageFeeExpected struct {
	TotalWithoutTax       *uint64  `json:"total_without_tax"`
	TaxAmount             *uint64  `json:"tax_amount"`
	TotalWithTax          *uint64  `json:"total_with_tax"`
	LowCostSpecialApplied *bool    `json:"low_cost_special_applied"`
	BreakdownResults      []uint64 `json:"breakdown_results"`
}

type brokerageFeeCase struct {
	ID          string               `json:"id"`
	Description string               `json:"description"`
	Input       brokerageFeeInput    `json:"input"`
	Expected    brokerageFeeExpected `json:"expected"`
}

type realEstateFixtures struct {
	BrokerageFee []brokerageFeeCase `json:"brokerage_fee"`
}

type incomeTaxInput struct {
	TaxableIncome          uint64 `json:"taxable_income"`
	Year                   int    `json:"year"`
	Month                  int    `json:"month"`
	Day                    int    `json:"day"`
	ApplyReconstructionTax bool   `json:"apply_reconstruction_tax"`
}

type incomeTaxExpected struct {
	BaseTax                  uint64 `json:"base_tax"`
	ReconstructionTax        uint64 `json:"reconstruction_tax"`
	TotalTax                 uint64 `json:"total_tax"`
	ReconstructionTaxApplied bool   `json:"reconstruction_tax_applied"`
}

type incomeTaxCase struct {
	ID          string            `json:"id"`
	Description string            `json:"description"`
	Input       incomeTaxInput    `json:"input"`
	Expected    incomeTaxExpected `json:"expected"`
}

type incomeTaxFixtures struct {
	IncomeTax []incomeTaxCase `json:"income_tax"`
}

type stampTaxInput struct {
	ContractAmount          uint64 `json:"contract_amount"`
	Year                    int    `json:"year"`
	Month                   int    `json:"month"`
	Day                     int    `json:"day"`
	IsReducedRateApplicable bool   `json:"is_reduced_rate_applicable"`
}

type stampTaxExpected struct {
	TaxAmount          uint64 `json:"tax_amount"`
	ReducedRateApplied bool   `json:"reduced_rate_applied"`
}

type stampTaxCase struct {
	ID          string           `json:"id"`
	Description string           `json:"description"`
	Input       stampTaxInput    `json:"input"`
	Expected    stampTaxExpected `json:"expected"`
}

type stampTaxFixtures struct {
	StampTax []stampTaxCase `json:"stamp_tax"`
}

type consumptionTaxInput struct {
	Amount       uint64 `json:"amount"`
	Year         int    `json:"year"`
	Month        int    `json:"month"`
	Day          int    `json:"day"`
	IsReducedRate bool  `json:"is_reduced_rate"`
}

type consumptionTaxExpected struct {
	TaxAmount        uint64 `json:"tax_amount"`
	AmountWithTax    uint64 `json:"amount_with_tax"`
	AmountWithoutTax uint64 `json:"amount_without_tax"`
	AppliedRateNumer uint64 `json:"applied_rate_numer"`
	AppliedRateDenom uint64 `json:"applied_rate_denom"`
	IsReducedRate    bool   `json:"is_reduced_rate"`
}

type consumptionTaxCase struct {
	ID          string                 `json:"id"`
	Description string                 `json:"description"`
	Input       consumptionTaxInput    `json:"input"`
	Expected    consumptionTaxExpected `json:"expected"`
}

type consumptionTaxFixtures struct {
	ConsumptionTax []consumptionTaxCase `json:"consumption_tax"`
}

// ─── フィクスチャ読み込み ────────────────────────────────────────────────────

func loadRealEstateFixtures(t *testing.T) realEstateFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/real_estate.json")
	if err != nil {
		t.Fatalf("failed to read real_estate.json: %v", err)
	}
	var f realEstateFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse real_estate.json: %v", err)
	}
	return f
}

func loadIncomeTaxFixtures(t *testing.T) incomeTaxFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/income_tax.json")
	if err != nil {
		t.Fatalf("failed to read income_tax.json: %v", err)
	}
	var f incomeTaxFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse income_tax.json: %v", err)
	}
	return f
}

func loadStampTaxFixtures(t *testing.T) stampTaxFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/stamp_tax.json")
	if err != nil {
		t.Fatalf("failed to read stamp_tax.json: %v", err)
	}
	var f stampTaxFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse stamp_tax.json: %v", err)
	}
	return f
}

func loadConsumptionTaxFixtures(t *testing.T) consumptionTaxFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/consumption_tax.json")
	if err != nil {
		t.Fatalf("failed to read consumption_tax.json: %v", err)
	}
	var f consumptionTaxFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse consumption_tax.json: %v", err)
	}
	return f
}

// ─── 不動産: データ駆動テスト ─────────────────────────────────────────────────

func TestBrokerageFee(t *testing.T) {
	fixtures := loadRealEstateFixtures(t)

	for _, tc := range fixtures.BrokerageFee {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcBrokerageFee(
				tc.Input.Price,
				tc.Input.Year, tc.Input.Month, tc.Input.Day,
				tc.Input.IsLowCostVacantHouse,
				tc.Input.IsSeller,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			exp := tc.Expected
			if exp.TotalWithoutTax != nil {
				if result.TotalWithoutTax != *exp.TotalWithoutTax {
					t.Errorf("TotalWithoutTax: got %d, want %d", result.TotalWithoutTax, *exp.TotalWithoutTax)
				}
			}
			if exp.TaxAmount != nil {
				if result.TaxAmount != *exp.TaxAmount {
					t.Errorf("TaxAmount: got %d, want %d", result.TaxAmount, *exp.TaxAmount)
				}
			}
			if exp.TotalWithTax != nil {
				if result.TotalWithTax != *exp.TotalWithTax {
					t.Errorf("TotalWithTax: got %d, want %d", result.TotalWithTax, *exp.TotalWithTax)
				}
			}
			if exp.LowCostSpecialApplied != nil {
				if result.LowCostSpecialApplied != *exp.LowCostSpecialApplied {
					t.Errorf("LowCostSpecialApplied: got %v, want %v", result.LowCostSpecialApplied, *exp.LowCostSpecialApplied)
				}
			}
			if exp.BreakdownResults != nil {
				if len(result.Breakdown) != len(exp.BreakdownResults) {
					t.Fatalf("Breakdown length: got %d, want %d", len(result.Breakdown), len(exp.BreakdownResults))
				}
				for i, want := range exp.BreakdownResults {
					if result.Breakdown[i].Result != want {
						t.Errorf("Breakdown[%d].Result: got %d, want %d", i, result.Breakdown[i].Result, want)
					}
				}
			}
		})
	}
}

// ─── 所得税: データ駆動テスト ─────────────────────────────────────────────────

func TestIncomeTax(t *testing.T) {
	fixtures := loadIncomeTaxFixtures(t)

	for _, tc := range fixtures.IncomeTax {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcIncomeTax(
				tc.Input.TaxableIncome,
				tc.Input.Year, tc.Input.Month, tc.Input.Day,
				tc.Input.ApplyReconstructionTax,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			exp := tc.Expected
			if result.BaseTax != exp.BaseTax {
				t.Errorf("BaseTax: got %d, want %d", result.BaseTax, exp.BaseTax)
			}
			if result.ReconstructionTax != exp.ReconstructionTax {
				t.Errorf("ReconstructionTax: got %d, want %d", result.ReconstructionTax, exp.ReconstructionTax)
			}
			if result.TotalTax != exp.TotalTax {
				t.Errorf("TotalTax: got %d, want %d", result.TotalTax, exp.TotalTax)
			}
			if result.ReconstructionTaxApplied != exp.ReconstructionTaxApplied {
				t.Errorf("ReconstructionTaxApplied: got %v, want %v", result.ReconstructionTaxApplied, exp.ReconstructionTaxApplied)
			}
		})
	}
}

// ─── 言語固有テスト ──────────────────────────────────────────────────────────

func TestBrokerageFee_ErrorDateOutOfRange(t *testing.T) {
	// 2018年以前はカバー範囲外（2018-01-01 が施行日のため 2017-12-31 はエラー）
	_, err := jlawcore.CalcBrokerageFee(5_000_000, 2017, 12, 31, false, false)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
	if !strings.Contains(err.Error(), "2017-12-31") {
		t.Errorf("error message should contain the invalid date, got: %s", err.Error())
	}
}

func TestBrokerageFee_BreakdownFields(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false, false)
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

func TestIncomeTax_ErrorDateOutOfRange(t *testing.T) {
	_, err := jlawcore.CalcIncomeTax(5_000_000, 2014, 12, 31, true)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
}

func TestIncomeTax_BreakdownFields(t *testing.T) {
	result, err := jlawcore.CalcIncomeTax(5_000_000, 2024, 1, 1, true)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(result.Breakdown) == 0 {
		t.Fatal("Breakdown must not be empty")
	}
	for _, step := range result.Breakdown {
		if step.Label == "" {
			t.Error("IncomeTaxStep.Label must not be empty")
		}
		if step.RateDenom == 0 {
			t.Error("IncomeTaxStep.RateDenom must not be zero")
		}
	}
}

// ─── 印紙税: データ駆動テスト ─────────────────────────────────────────────────

func TestStampTax(t *testing.T) {
	fixtures := loadStampTaxFixtures(t)

	for _, tc := range fixtures.StampTax {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcStampTax(
				tc.Input.ContractAmount,
				tc.Input.Year, tc.Input.Month, tc.Input.Day,
				tc.Input.IsReducedRateApplicable,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			exp := tc.Expected
			if result.TaxAmount != exp.TaxAmount {
				t.Errorf("TaxAmount: got %d, want %d", result.TaxAmount, exp.TaxAmount)
			}
			if result.ReducedRateApplied != exp.ReducedRateApplied {
				t.Errorf("ReducedRateApplied: got %v, want %v", result.ReducedRateApplied, exp.ReducedRateApplied)
			}
		})
	}
}

// ─── 消費税: データ駆動テスト ─────────────────────────────────────────────────

func TestConsumptionTax(t *testing.T) {
	fixtures := loadConsumptionTaxFixtures(t)

	for _, tc := range fixtures.ConsumptionTax {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcConsumptionTax(
				tc.Input.Amount,
				tc.Input.Year, tc.Input.Month, tc.Input.Day,
				tc.Input.IsReducedRate,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			exp := tc.Expected
			if result.TaxAmount != exp.TaxAmount {
				t.Errorf("TaxAmount: got %d, want %d", result.TaxAmount, exp.TaxAmount)
			}
			if result.AmountWithTax != exp.AmountWithTax {
				t.Errorf("AmountWithTax: got %d, want %d", result.AmountWithTax, exp.AmountWithTax)
			}
			if result.AmountWithoutTax != exp.AmountWithoutTax {
				t.Errorf("AmountWithoutTax: got %d, want %d", result.AmountWithoutTax, exp.AmountWithoutTax)
			}
			if result.AppliedRateNumer != exp.AppliedRateNumer {
				t.Errorf("AppliedRateNumer: got %d, want %d", result.AppliedRateNumer, exp.AppliedRateNumer)
			}
			if result.AppliedRateDenom != exp.AppliedRateDenom {
				t.Errorf("AppliedRateDenom: got %d, want %d", result.AppliedRateDenom, exp.AppliedRateDenom)
			}
			if result.IsReducedRate != exp.IsReducedRate {
				t.Errorf("IsReducedRate: got %v, want %v", result.IsReducedRate, exp.IsReducedRate)
			}
		})
	}
}

// ─── 消費税: 言語固有テスト ────────────────────────────────────────────────────

func TestConsumptionTax_ErrorReducedRateWithoutSupport(t *testing.T) {
	// 2016年は標準8%のみ、軽減税率は存在しないためエラー
	_, err := jlawcore.CalcConsumptionTax(100_000, 2016, 1, 1, true)
	if err == nil {
		t.Fatal("expected error for reduced rate without support, got nil")
	}
}

func TestConsumptionTax_BeforeIntroductionNoTax(t *testing.T) {
	// 消費税導入前（1988年）は税額ゼロで正常終了
	result, err := jlawcore.CalcConsumptionTax(100_000, 1988, 1, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.TaxAmount != 0 {
		t.Errorf("TaxAmount: got %d, want 0", result.TaxAmount)
	}
	if result.AmountWithTax != 100_000 {
		t.Errorf("AmountWithTax: got %d, want 100000", result.AmountWithTax)
	}
}

// ─── 印紙税: 言語固有テスト ────────────────────────────────────────────────────

func TestStampTax_ErrorDateOutOfRange(t *testing.T) {
	_, err := jlawcore.CalcStampTax(5_000_000, 2014, 3, 31, false)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
}

func TestStampTax_BracketLabelPresent(t *testing.T) {
	result, err := jlawcore.CalcStampTax(5_000_000, 2024, 8, 1, false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.BracketLabel == "" {
		t.Error("BracketLabel must not be empty")
	}
}
