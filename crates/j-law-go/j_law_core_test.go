package jlawcore_test

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
	"time"

	jlawcore "github.com/kmoyashi/j-law-go"
)

// ─── 日付ユーティリティ ──────────────────────────────────────────────────────

func parseDate(t *testing.T, date string) time.Time {
	t.Helper()
	d, err := time.Parse("2006-01-02", date)
	if err != nil {
		t.Fatalf("invalid date: %s", date)
	}
	return d
}

// ─── フィクスチャ型定義 ──────────────────────────────────────────────────────

type brokerageFeeInput struct {
	Price                uint64 `json:"price"`
	Date                 string `json:"date"`
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
	Date                   string `json:"date"`
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
	DocumentKind            string `json:"document_kind"`
	ContractAmount          uint64 `json:"contract_amount"`
	Date                    string `json:"date"`
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

func parseStampTaxDocumentKind(t *testing.T, value string) jlawcore.StampTaxDocumentKind {
	t.Helper()
	switch value {
	case string(jlawcore.StampTaxDocumentRealEstateTransfer):
		return jlawcore.StampTaxDocumentRealEstateTransfer
	case string(jlawcore.StampTaxDocumentConstructionContract):
		return jlawcore.StampTaxDocumentConstructionContract
	default:
		t.Fatalf("invalid stamp tax document kind fixture: %s", value)
		return ""
	}
}

type consumptionTaxInput struct {
	Amount        uint64 `json:"amount"`
	Date          string `json:"date"`
	IsReducedRate bool   `json:"is_reduced_rate"`
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

type withholdingTaxInput struct {
	PaymentAmount                  uint64 `json:"payment_amount"`
	SeparatedConsumptionTaxAmount  uint64 `json:"separated_consumption_tax_amount"`
	Date                           string `json:"date"`
	Category                       string `json:"category"`
	IsSubmissionPrize              bool   `json:"is_submission_prize"`
}

type withholdingTaxExpected struct {
	TaxablePaymentAmount    uint64 `json:"taxable_payment_amount"`
	TaxAmount               uint64 `json:"tax_amount"`
	NetPaymentAmount        uint64 `json:"net_payment_amount"`
	SubmissionPrizeExempted bool   `json:"submission_prize_exempted"`
}

type withholdingTaxCase struct {
	ID          string                 `json:"id"`
	Description string                 `json:"description"`
	Input       withholdingTaxInput    `json:"input"`
	Expected    withholdingTaxExpected `json:"expected"`
}

type withholdingTaxFixtures struct {
	WithholdingTax []withholdingTaxCase `json:"withholding_tax"`
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

func loadWithholdingTaxFixtures(t *testing.T) withholdingTaxFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/withholding_tax.json")
	if err != nil {
		t.Fatalf("failed to read withholding_tax.json: %v", err)
	}
	var f withholdingTaxFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse withholding_tax.json: %v", err)
	}
	return f
}

func parseWithholdingTaxCategory(t *testing.T, category string) jlawcore.WithholdingTaxCategory {
	t.Helper()
	switch category {
	case "manuscript_and_lecture":
		return jlawcore.WithholdingTaxCategoryManuscriptAndLecture
	case "professional_fee":
		return jlawcore.WithholdingTaxCategoryProfessionalFee
	case "exclusive_contract_fee":
		return jlawcore.WithholdingTaxCategoryExclusiveContractFee
	default:
		t.Fatalf("unknown withholding tax category: %s", category)
		return 0
	}
}

// ─── 不動産: データ駆動テスト ─────────────────────────────────────────────────

func TestBrokerageFee(t *testing.T) {
	fixtures := loadRealEstateFixtures(t)

	for _, tc := range fixtures.BrokerageFee {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcBrokerageFee(
				tc.Input.Price,
				parseDate(t, tc.Input.Date),
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
				parseDate(t, tc.Input.Date),
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
	// 1970-12-01 施行のため、それ以前の日付はエラー
	_, err := jlawcore.CalcBrokerageFee(5_000_000, time.Date(1970, time.November, 30, 0, 0, 0, 0, time.UTC), false, false)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
	if !strings.Contains(err.Error(), "1970-11-30") {
		t.Errorf("error message should contain the invalid date, got: %s", err.Error())
	}
}

func TestBrokerageFee_BreakdownFields(t *testing.T) {
	result, err := jlawcore.CalcBrokerageFee(5_000_000, time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC), false, false)
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
	_, err := jlawcore.CalcIncomeTax(5_000_000, time.Date(1988, time.December, 31, 0, 0, 0, 0, time.UTC), true)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
}

func TestIncomeTax_BreakdownFields(t *testing.T) {
	result, err := jlawcore.CalcIncomeTax(5_000_000, time.Date(2024, time.January, 1, 0, 0, 0, 0, time.UTC), true)
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
			result, err := jlawcore.CalcStampTaxWithDocumentKind(
				tc.Input.ContractAmount,
				parseDate(t, tc.Input.Date),
				tc.Input.IsReducedRateApplicable,
				parseStampTaxDocumentKind(t, tc.Input.DocumentKind),
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
				parseDate(t, tc.Input.Date),
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
	_, err := jlawcore.CalcConsumptionTax(100_000, time.Date(2016, time.January, 1, 0, 0, 0, 0, time.UTC), true)
	if err == nil {
		t.Fatal("expected error for reduced rate without support, got nil")
	}
}

func TestConsumptionTax_BeforeIntroductionNoTax(t *testing.T) {
	// 消費税導入前（1988年）は税額ゼロで正常終了
	result, err := jlawcore.CalcConsumptionTax(100_000, time.Date(1988, time.January, 1, 0, 0, 0, 0, time.UTC), false)
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

// ─── 源泉徴収: データ駆動テスト ───────────────────────────────────────────────

func TestWithholdingTax(t *testing.T) {
	fixtures := loadWithholdingTaxFixtures(t)

	for _, tc := range fixtures.WithholdingTax {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcWithholdingTax(
				tc.Input.PaymentAmount,
				parseDate(t, tc.Input.Date),
				parseWithholdingTaxCategory(t, tc.Input.Category),
				tc.Input.IsSubmissionPrize,
				tc.Input.SeparatedConsumptionTaxAmount,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			exp := tc.Expected
			if result.TaxablePaymentAmount != exp.TaxablePaymentAmount {
				t.Errorf("TaxablePaymentAmount: got %d, want %d", result.TaxablePaymentAmount, exp.TaxablePaymentAmount)
			}
			if result.TaxAmount != exp.TaxAmount {
				t.Errorf("TaxAmount: got %d, want %d", result.TaxAmount, exp.TaxAmount)
			}
			if result.NetPaymentAmount != exp.NetPaymentAmount {
				t.Errorf("NetPaymentAmount: got %d, want %d", result.NetPaymentAmount, exp.NetPaymentAmount)
			}
			if result.SubmissionPrizeExempted != exp.SubmissionPrizeExempted {
				t.Errorf("SubmissionPrizeExempted: got %v, want %v", result.SubmissionPrizeExempted, exp.SubmissionPrizeExempted)
			}
		})
	}
}

// ─── 印紙税: 言語固有テスト ────────────────────────────────────────────────────

func TestStampTax_ErrorDateOutOfRange(t *testing.T) {
	_, err := jlawcore.CalcStampTax(5_000_000, time.Date(2014, time.March, 31, 0, 0, 0, 0, time.UTC), false)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
}

func TestStampTax_BracketLabelPresent(t *testing.T) {
	result, err := jlawcore.CalcStampTax(5_000_000, time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC), false)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.BracketLabel == "" {
		t.Error("BracketLabel must not be empty")
	}
}

func TestStampTax_InvalidDocumentKind(t *testing.T) {
	_, err := jlawcore.CalcStampTaxWithDocumentKind(
		5_000_000,
		time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC),
		false,
		jlawcore.StampTaxDocumentKind("invalid_kind"),
	)
	if err == nil {
		t.Fatal("expected error for invalid document kind, got nil")
	}
}

func TestWithholdingTax_ErrorDateOutOfRange(t *testing.T) {
	_, err := jlawcore.CalcWithholdingTax(
		100_000,
		time.Date(2012, time.December, 31, 0, 0, 0, 0, time.UTC),
		jlawcore.WithholdingTaxCategoryManuscriptAndLecture,
		false,
		0,
	)
	if err == nil {
		t.Fatal("expected error for date out of range, got nil")
	}
}

func TestWithholdingTax_BreakdownFields(t *testing.T) {
	result, err := jlawcore.CalcWithholdingTax(
		1_500_000,
		time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC),
		jlawcore.WithholdingTaxCategoryProfessionalFee,
		false,
		0,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(result.Breakdown) != 2 {
		t.Fatalf("Breakdown length: got %d, want 2", len(result.Breakdown))
	}
	if result.Breakdown[0].Label == "" {
		t.Error("BreakdownStep.Label must not be empty")
	}
}
