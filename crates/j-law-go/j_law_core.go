// Package jlawcore は、宅建業法第46条に基づく媒介報酬計算を提供する。
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
//     WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
//
// エラー: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcBrokerageFee(
	price uint64,
	year, month, day int,
	isLowCostVacantHouse bool,
) (*BrokerageFeeResult, error) {
	var cResult C.JLawBrokerageFeeResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	isLowCost := C.int(0)
	if isLowCostVacantHouse {
		isLowCost = 1
	}

	ret := C.j_law_calc_brokerage_fee(
		C.uint64_t(price),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		isLowCost,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoResult(&cResult), nil
}

// ─── 内部変換 ───────────────────────────────────────────────────────────────────

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
