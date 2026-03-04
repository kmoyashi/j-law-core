// Package jlawcore は、日本の法令に基づく各種計算を提供する。
//
// j-law-cgo（Rust staticlib）を CGo 経由で静的リンクしている。
// 使用前に `make build-rust` を実行して静的ライブラリをビルドすること。
//
// 使用例:
//
//	result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(result.TotalWithTax) // 231000
//
//	taxResult, err := jlawcore.CalcIncomeTax(5_000_000, 2024, 1, 1, true)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(taxResult.TotalTax) // 584500
package jlawcore

// #cgo CFLAGS: -I${SRCDIR}/../j-law-cgo
// #cgo darwin LDFLAGS: ${SRCDIR}/../../target/debug/libj_law_cgo.a -framework Security -framework CoreFoundation
// #cgo linux  LDFLAGS: ${SRCDIR}/../../target/debug/libj_law_cgo.a -ldl -lpthread -lm
// #include "j_law_cgo.h"
// #include <stdlib.h>
import "C"
import (
	"errors"
	"unsafe"
)

// ─── Go 公開型 ──────────────────────────────────────────────────────────────────

// BreakdownStep は 1 ティアの計算内訳を表す。
type BreakdownStep struct {
	// Label はティアの名称（例: "tier1"）。
	Label string
	// BaseAmount はティア対象金額（円）。
	BaseAmount uint64
	RateNumer  uint64
	RateDenom  uint64
	// Result はティア計算結果（円・端数切捨て済み）。
	Result uint64
}

// BrokerageFeeResult は媒介報酬の計算結果を表す。
type BrokerageFeeResult struct {
	// TotalWithoutTax は税抜合計額（円）。
	TotalWithoutTax uint64
	// TotalWithTax は税込合計額（円）。
	TotalWithTax uint64
	// TaxAmount は消費税額（円）。
	TaxAmount uint64
	// LowCostSpecialApplied は低廉な空き家特例が適用されたかを示す。
	LowCostSpecialApplied bool
	// Breakdown は各ティアの計算内訳。
	Breakdown []BreakdownStep
}

// ─── Go 公開関数 ────────────────────────────────────────────────────────────────

// CalcBrokerageFee は宅建業法第46条に基づく媒介報酬を計算する。
//
// 法的根拠: 宅地建物取引業法 第46条第1項 / 国土交通省告示
//
// 引数:
//   - price: 売買価格（円）
//   - year, month, day: 基準日
//   - isLowCostVacantHouse: 低廉な空き家特例フラグ
//     WARNING: 対象物件が「低廉な空き家等」に該当するかの事実認定は呼び出し元の責任。
//   - isSeller: 売主側フラグ（2018年1月1日〜2024年6月30日の低廉特例は売主のみ適用）
//     WARNING: 売主・買主の事実認定は呼び出し元の責任。
//
// エラー: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcBrokerageFee(
	price uint64,
	year, month, day int,
	isLowCostVacantHouse bool,
	isSeller bool,
) (*BrokerageFeeResult, error) {
	var cResult C.JLawBrokerageFeeResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	isLowCost := C.int(0)
	if isLowCostVacantHouse {
		isLowCost = 1
	}
	isSellerInt := C.int(0)
	if isSeller {
		isSellerInt = 1
	}

	ret := C.j_law_calc_brokerage_fee(
		C.uint64_t(price),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		isLowCost,
		isSellerInt,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoResult(&cResult), nil
}

// ─── 所得税 Go 公開型 ───────────────────────────────────────────────────────────

// IncomeTaxStep は所得税の計算内訳（速算表の適用結果）を表す。
type IncomeTaxStep struct {
	// Label は内訳の名称。
	Label string
	// TaxableIncome は課税所得金額（円）。
	TaxableIncome uint64
	RateNumer     uint64
	RateDenom     uint64
	// Deduction は速算表の控除額（円）。
	Deduction uint64
	// Result は算出税額（円）。
	Result uint64
}

// IncomeTaxResult は所得税の計算結果を表す。
type IncomeTaxResult struct {
	// BaseTax は基準所得税額（円）。
	BaseTax uint64
	// ReconstructionTax は復興特別所得税額（円）。
	ReconstructionTax uint64
	// TotalTax は申告納税額（円・100円未満切り捨て）。
	TotalTax uint64
	// ReconstructionTaxApplied は復興特別所得税が適用されたかを示す。
	ReconstructionTaxApplied bool
	// Breakdown は計算内訳。
	Breakdown []IncomeTaxStep
}

// ─── 所得税 Go 公開関数 ─────────────────────────────────────────────────────────

// CalcIncomeTax は所得税法第89条に基づく所得税額を計算する。
//
// 法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条
//
// 引数:
//   - taxableIncome: 課税所得金額（円）
//   - year, month, day: 基準日
//   - applyReconstructionTax: 復興特別所得税を適用するか
//
// エラー: 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcIncomeTax(
	taxableIncome uint64,
	year, month, day int,
	applyReconstructionTax bool,
) (*IncomeTaxResult, error) {
	var cResult C.JLawIncomeTaxResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	applyFlag := C.int(0)
	if applyReconstructionTax {
		applyFlag = 1
	}

	ret := C.j_law_calc_income_tax(
		C.uint64_t(taxableIncome),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		applyFlag,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoIncomeTaxResult(&cResult), nil
}

// ─── 内部変換 ───────────────────────────────────────────────────────────────────

// toGoIncomeTaxResult は所得税の C 構造体を Go 構造体に変換する。
func toGoIncomeTaxResult(c *C.JLawIncomeTaxResult) *IncomeTaxResult {
	breakdownLen := int(c.breakdown_len)
	breakdown := make([]IncomeTaxStep, breakdownLen)
	for i := 0; i < breakdownLen; i++ {
		step := &c.breakdown[i]
		breakdown[i] = IncomeTaxStep{
			Label:         C.GoString(&step.label[0]),
			TaxableIncome: uint64(step.taxable_income),
			RateNumer:     uint64(step.rate_numer),
			RateDenom:     uint64(step.rate_denom),
			Deduction:     uint64(step.deduction),
			Result:        uint64(step.result),
		}
	}

	return &IncomeTaxResult{
		BaseTax:                  uint64(c.base_tax),
		ReconstructionTax:        uint64(c.reconstruction_tax),
		TotalTax:                 uint64(c.total_tax),
		ReconstructionTaxApplied: c.reconstruction_tax_applied != 0,
		Breakdown:                breakdown,
	}
}

// toGoResult は C 構造体を Go 構造体に変換する。
func toGoResult(c *C.JLawBrokerageFeeResult) *BrokerageFeeResult {
	breakdownLen := int(c.breakdown_len)
	breakdown := make([]BreakdownStep, breakdownLen)
	for i := 0; i < breakdownLen; i++ {
		step := &c.breakdown[i]
		breakdown[i] = BreakdownStep{
			Label:      C.GoString(&step.label[0]),
			BaseAmount: uint64(step.base_amount),
			RateNumer:  uint64(step.rate_numer),
			RateDenom:  uint64(step.rate_denom),
			Result:     uint64(step.result),
		}
	}

	return &BrokerageFeeResult{
		TotalWithoutTax:       uint64(c.total_without_tax),
		TotalWithTax:          uint64(c.total_with_tax),
		TaxAmount:             uint64(c.tax_amount),
		LowCostSpecialApplied: c.low_cost_special_applied != 0,
		Breakdown:             breakdown,
	}
}

// ─── 消費税 Go 公開型 ───────────────────────────────────────────────────────────

// ConsumptionTaxResult は消費税の計算結果を表す。
type ConsumptionTaxResult struct {
	// TaxAmount は消費税額（円）。
	TaxAmount uint64
	// AmountWithTax は税込金額（円）。
	AmountWithTax uint64
	// AmountWithoutTax は税抜金額（円）。
	AmountWithoutTax uint64
	// AppliedRateNumer は適用税率の分子。
	AppliedRateNumer uint64
	// AppliedRateDenom は適用税率の分母。
	AppliedRateDenom uint64
	// IsReducedRate は軽減税率が適用されたかを示す。
	IsReducedRate bool
}

// ─── 消費税 Go 公開関数 ─────────────────────────────────────────────────────────

// CalcConsumptionTax は消費税法第29条に基づく消費税額を計算する。
//
// 法的根拠: 消費税法 第29条（税率）
//
// 引数:
//   - amount: 課税標準額（税抜き・円）
//   - year, month, day: 基準日
//   - isReducedRate: 軽減税率フラグ（2019-10-01以降の飲食料品・新聞等）
//     WARNING: 対象が軽減税率の適用要件を満たすかの事実認定は呼び出し元の責任。
//
// エラー: 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合。
func CalcConsumptionTax(
	amount uint64,
	year, month, day int,
	isReducedRate bool,
) (*ConsumptionTaxResult, error) {
	var cResult C.JLawConsumptionTaxResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	isReduced := C.int(0)
	if isReducedRate {
		isReduced = 1
	}

	ret := C.j_law_calc_consumption_tax(
		C.uint64_t(amount),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		isReduced,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoConsumptionTaxResult(&cResult), nil
}

// toGoConsumptionTaxResult は消費税の C 構造体を Go 構造体に変換する。
func toGoConsumptionTaxResult(c *C.JLawConsumptionTaxResult) *ConsumptionTaxResult {
	return &ConsumptionTaxResult{
		TaxAmount:        uint64(c.tax_amount),
		AmountWithTax:    uint64(c.amount_with_tax),
		AmountWithoutTax: uint64(c.amount_without_tax),
		AppliedRateNumer: uint64(c.applied_rate_numer),
		AppliedRateDenom: uint64(c.applied_rate_denom),
		IsReducedRate:    c.is_reduced_rate != 0,
	}
}

// ─── 印紙税 Go 公開型 ───────────────────────────────────────────────────────────

// StampTaxResult は印紙税の計算結果を表す。
type StampTaxResult struct {
	// TaxAmount は印紙税額（円）。
	TaxAmount uint64
	// BracketLabel は適用されたブラケットの表示名。
	BracketLabel string
	// ReducedRateApplied は軽減税率が適用されたかを示す。
	ReducedRateApplied bool
}

// ─── 印紙税 Go 公開関数 ─────────────────────────────────────────────────────────

// CalcStampTax は印紙税法 別表第一に基づく印紙税額を計算する。
//
// 法的根拠: 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条
//
// 引数:
//   - contractAmount: 契約金額（円）
//   - year, month, day: 契約書作成日
//   - isReducedRateApplicable: 軽減税率適用フラグ
//     WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任。
//
// エラー: 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcStampTax(
	contractAmount uint64,
	year, month, day int,
	isReducedRateApplicable bool,
) (*StampTaxResult, error) {
	var cResult C.JLawStampTaxResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	isReduced := C.int(0)
	if isReducedRateApplicable {
		isReduced = 1
	}

	ret := C.j_law_calc_stamp_tax(
		C.uint64_t(contractAmount),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		isReduced,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return &StampTaxResult{
		TaxAmount:          uint64(cResult.tax_amount),
		BracketLabel:       C.GoString(&cResult.bracket_label[0]),
		ReducedRateApplied: cResult.reduced_rate_applied != 0,
	}, nil
}
