// Package jlawcore は、日本の法令に基づく各種計算を提供する。
//
// j-law-c-ffi が提供する C FFI を CGo 経由で利用している。
// 使用前に `make build-rust` を実行して静的ライブラリをビルドすること。
//
// 使用例:
//
//	date := time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC)
//	result, err := jlawcore.CalcBrokerageFee(5_000_000, date, false, false)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(result.TotalWithTax) // 231000
//
//	taxDate := time.Date(2024, time.January, 1, 0, 0, 0, 0, time.UTC)
//	taxResult, err := jlawcore.CalcIncomeTax(5_000_000, taxDate, true)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(taxResult.TotalTax) // 584500
package jlawcore

// #cgo CFLAGS: -I${SRCDIR}/../j-law-c-ffi
// #cgo darwin LDFLAGS: ${SRCDIR}/../../target/debug/libj_law_c_ffi.a -framework Security -framework CoreFoundation
// #cgo linux  LDFLAGS: ${SRCDIR}/../../target/debug/libj_law_c_ffi.a -ldl -lpthread -lm
// #include "j_law_c_ffi.h"
// #include <stdlib.h>
import "C"
import (
	"errors"
	"fmt"
	"time"
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
//   - date: 基準日
//   - isLowCostVacantHouse: 低廉な空き家特例フラグ
//     WARNING: 対象物件が「低廉な空き家等」に該当するかの事実認定は呼び出し元の責任。
//   - isSeller: 売主側フラグ（2018年1月1日〜2024年6月30日の低廉特例は売主のみ適用）
//     WARNING: 売主・買主の事実認定は呼び出し元の責任。
//
// エラー: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcBrokerageFee(
	price uint64,
	date time.Time,
	isLowCostVacantHouse bool,
	isSeller bool,
) (*BrokerageFeeResult, error) {
	year := date.Year()
	month := int(date.Month())
	day := date.Day()
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

// IncomeDeductionLine は所得控除の内訳1行を表す。
type IncomeDeductionLine struct {
	Kind   uint32
	Label  string
	Amount uint64
}

// SpouseDeductionInput は配偶者控除の入力を表す。
type SpouseDeductionInput struct {
	SpouseTotalIncomeAmount uint64
	IsSameHousehold         bool
	IsElderly               bool
}

// DependentDeductionInput は扶養控除の入力を表す。
type DependentDeductionInput struct {
	GeneralCount           uint64
	SpecificCount          uint64
	ElderlyCohabitingCount uint64
	ElderlyOtherCount      uint64
}

// MedicalDeductionInput は医療費控除の入力を表す。
type MedicalDeductionInput struct {
	MedicalExpensePaid uint64
	ReimbursedAmount   uint64
}

// LifeInsuranceDeductionInput は生命保険料控除の入力を表す。
type LifeInsuranceDeductionInput struct {
	NewGeneralPaidAmount           uint64
	NewIndividualPensionPaidAmount uint64
	NewCareMedicalPaidAmount       uint64
	OldGeneralPaidAmount           uint64
	OldIndividualPensionPaidAmount uint64
}

// DonationDeductionInput は寄附金控除の入力を表す。
type DonationDeductionInput struct {
	QualifiedDonationAmount uint64
}

// IncomeDeductionInput は所得控除計算の入力を表す。
type IncomeDeductionInput struct {
	TotalIncomeAmount          uint64
	Date                       time.Time
	Spouse                     *SpouseDeductionInput
	Dependent                  DependentDeductionInput
	SocialInsurancePremiumPaid uint64
	Medical                    *MedicalDeductionInput
	LifeInsurance              *LifeInsuranceDeductionInput
	Donation                   *DonationDeductionInput
}

// IncomeDeductionResult は所得控除の計算結果を表す。
type IncomeDeductionResult struct {
	TotalIncomeAmount             uint64
	TotalDeductions               uint64
	TaxableIncomeBeforeTruncation uint64
	TaxableIncome                 uint64
	Breakdown                     []IncomeDeductionLine
}

// IncomeTaxAssessmentResult は所得控除から所得税額までの通し計算結果を表す。
type IncomeTaxAssessmentResult struct {
	Deductions *IncomeDeductionResult
	Tax        *IncomeTaxResult
}

// ─── 所得税 Go 公開関数 ─────────────────────────────────────────────────────────

// CalcIncomeTax は所得税法第89条に基づく所得税額を計算する。
//
// 法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条
//
// 引数:
//   - taxableIncome: 課税所得金額（円）
//   - date: 基準日
//   - applyReconstructionTax: 復興特別所得税を適用するか
//
// エラー: 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合。
func CalcIncomeTax(
	taxableIncome uint64,
	date time.Time,
	applyReconstructionTax bool,
) (*IncomeTaxResult, error) {
	year := date.Year()
	month := int(date.Month())
	day := date.Day()
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

// CalcIncomeDeductions は総所得金額等から所得控除を計算し、課税所得金額までを返す。
func CalcIncomeDeductions(
	input IncomeDeductionInput,
) (*IncomeDeductionResult, error) {
	var cResult C.JLawIncomeDeductionResult
	cInput := toCIncomeDeductionInput(input)
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	ret := C.j_law_calc_income_deductions(
		&cInput,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoIncomeDeductionResult(&cResult), nil
}

// CalcIncomeTaxAssessment は所得控除から所得税額までを通しで計算する。
func CalcIncomeTaxAssessment(
	input IncomeDeductionInput,
	applyReconstructionTax bool,
) (*IncomeTaxAssessmentResult, error) {
	var cResult C.JLawIncomeTaxAssessmentResult
	cInput := toCIncomeDeductionInput(input)
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	applyFlag := C.int(0)
	if applyReconstructionTax {
		applyFlag = 1
	}

	ret := C.j_law_calc_income_tax_assessment(
		&cInput,
		applyFlag,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return &IncomeTaxAssessmentResult{
		Deductions: toGoIncomeDeductionResultFromAssessment(&cResult),
		Tax:        toGoIncomeTaxResultFromAssessment(&cResult),
	}, nil
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

func toGoIncomeDeductionResult(c *C.JLawIncomeDeductionResult) *IncomeDeductionResult {
	breakdownLen := int(c.breakdown_len)
	breakdown := make([]IncomeDeductionLine, breakdownLen)
	for i := 0; i < breakdownLen; i++ {
		line := &c.breakdown[i]
		breakdown[i] = IncomeDeductionLine{
			Kind:   uint32(line.kind),
			Label:  C.GoString(&line.label[0]),
			Amount: uint64(line.amount),
		}
	}

	return &IncomeDeductionResult{
		TotalIncomeAmount:             uint64(c.total_income_amount),
		TotalDeductions:               uint64(c.total_deductions),
		TaxableIncomeBeforeTruncation: uint64(c.taxable_income_before_truncation),
		TaxableIncome:                 uint64(c.taxable_income),
		Breakdown:                     breakdown,
	}
}

func toGoIncomeDeductionResultFromAssessment(
	c *C.JLawIncomeTaxAssessmentResult,
) *IncomeDeductionResult {
	breakdownLen := int(c.deduction_breakdown_len)
	breakdown := make([]IncomeDeductionLine, breakdownLen)
	for i := 0; i < breakdownLen; i++ {
		line := &c.deduction_breakdown[i]
		breakdown[i] = IncomeDeductionLine{
			Kind:   uint32(line.kind),
			Label:  C.GoString(&line.label[0]),
			Amount: uint64(line.amount),
		}
	}

	return &IncomeDeductionResult{
		TotalIncomeAmount:             uint64(c.total_income_amount),
		TotalDeductions:               uint64(c.total_deductions),
		TaxableIncomeBeforeTruncation: uint64(c.taxable_income_before_truncation),
		TaxableIncome:                 uint64(c.taxable_income),
		Breakdown:                     breakdown,
	}
}

func toGoIncomeTaxResultFromAssessment(
	c *C.JLawIncomeTaxAssessmentResult,
) *IncomeTaxResult {
	breakdownLen := int(c.tax_breakdown_len)
	breakdown := make([]IncomeTaxStep, breakdownLen)
	for i := 0; i < breakdownLen; i++ {
		step := &c.tax_breakdown[i]
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

func toCIncomeDeductionInput(input IncomeDeductionInput) C.JLawIncomeDeductionInput {
	year := input.Date.Year()
	month := int(input.Date.Month())
	day := input.Date.Day()

	cInput := C.JLawIncomeDeductionInput{
		total_income_amount:                C.uint64_t(input.TotalIncomeAmount),
		year:                               C.uint16_t(year),
		month:                              C.uint8_t(month),
		day:                                C.uint8_t(day),
		dependent_general_count:            C.uint64_t(input.Dependent.GeneralCount),
		dependent_specific_count:           C.uint64_t(input.Dependent.SpecificCount),
		dependent_elderly_cohabiting_count: C.uint64_t(input.Dependent.ElderlyCohabitingCount),
		dependent_elderly_other_count:      C.uint64_t(input.Dependent.ElderlyOtherCount),
		social_insurance_premium_paid:      C.uint64_t(input.SocialInsurancePremiumPaid),
	}

	if input.Spouse != nil {
		cInput.has_spouse = 1
		cInput.spouse_total_income_amount = C.uint64_t(input.Spouse.SpouseTotalIncomeAmount)
		if input.Spouse.IsSameHousehold {
			cInput.spouse_is_same_household = 1
		}
		if input.Spouse.IsElderly {
			cInput.spouse_is_elderly = 1
		}
	}
	if input.Medical != nil {
		cInput.has_medical = 1
		cInput.medical_expense_paid = C.uint64_t(input.Medical.MedicalExpensePaid)
		cInput.medical_reimbursed_amount = C.uint64_t(input.Medical.ReimbursedAmount)
	}
	if input.LifeInsurance != nil {
		cInput.has_life_insurance = 1
		cInput.life_new_general_paid_amount = C.uint64_t(input.LifeInsurance.NewGeneralPaidAmount)
		cInput.life_new_individual_pension_paid_amount = C.uint64_t(input.LifeInsurance.NewIndividualPensionPaidAmount)
		cInput.life_new_care_medical_paid_amount = C.uint64_t(input.LifeInsurance.NewCareMedicalPaidAmount)
		cInput.life_old_general_paid_amount = C.uint64_t(input.LifeInsurance.OldGeneralPaidAmount)
		cInput.life_old_individual_pension_paid_amount = C.uint64_t(input.LifeInsurance.OldIndividualPensionPaidAmount)
	}
	if input.Donation != nil {
		cInput.has_donation = 1
		cInput.donation_qualified_amount = C.uint64_t(input.Donation.QualifiedDonationAmount)
	}

	return cInput
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

// ─── 源泉徴収 Go 公開型 ─────────────────────────────────────────────────────────

// WithholdingTaxCategory は報酬・料金等の源泉徴収カテゴリを表す。
type WithholdingTaxCategory uint32

const (
	// WithholdingTaxCategoryManuscriptAndLecture は原稿料・講演料等。
	WithholdingTaxCategoryManuscriptAndLecture WithholdingTaxCategory = C.J_LAW_WITHHOLDING_TAX_CATEGORY_MANUSCRIPT_AND_LECTURE
	// WithholdingTaxCategoryProfessionalFee は税理士等の報酬・料金。
	WithholdingTaxCategoryProfessionalFee WithholdingTaxCategory = C.J_LAW_WITHHOLDING_TAX_CATEGORY_PROFESSIONAL_FEE
	// WithholdingTaxCategoryExclusiveContractFee は専属契約金。
	WithholdingTaxCategoryExclusiveContractFee WithholdingTaxCategory = C.J_LAW_WITHHOLDING_TAX_CATEGORY_EXCLUSIVE_CONTRACT_FEE
)

// WithholdingTaxResult は源泉徴収税額の計算結果を表す。
type WithholdingTaxResult struct {
	GrossPaymentAmount      uint64
	TaxablePaymentAmount    uint64
	TaxAmount               uint64
	NetPaymentAmount        uint64
	Category                WithholdingTaxCategory
	SubmissionPrizeExempted bool
	Breakdown               []BreakdownStep
}

// CalcWithholdingTax は所得税法第204条第1項に基づく報酬・料金等の源泉徴収税額を計算する。
func CalcWithholdingTax(
	paymentAmount uint64,
	date time.Time,
	category WithholdingTaxCategory,
	isSubmissionPrize bool,
	separatedConsumptionTaxAmount uint64,
) (*WithholdingTaxResult, error) {
	year := date.Year()
	month := int(date.Month())
	day := date.Day()
	var cResult C.JLawWithholdingTaxResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	submissionPrize := C.int(0)
	if isSubmissionPrize {
		submissionPrize = 1
	}

	ret := C.j_law_calc_withholding_tax(
		C.uint64_t(paymentAmount),
		C.uint64_t(separatedConsumptionTaxAmount),
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		C.uint32_t(category),
		submissionPrize,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	return toGoWithholdingTaxResult(&cResult), nil
}

func toGoWithholdingTaxResult(c *C.JLawWithholdingTaxResult) *WithholdingTaxResult {
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

	return &WithholdingTaxResult{
		GrossPaymentAmount:      uint64(c.gross_payment_amount),
		TaxablePaymentAmount:    uint64(c.taxable_payment_amount),
		TaxAmount:               uint64(c.tax_amount),
		NetPaymentAmount:        uint64(c.net_payment_amount),
		Category:                WithholdingTaxCategory(c.category),
		SubmissionPrizeExempted: c.submission_prize_exempted != 0,
		Breakdown:               breakdown,
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
//   - date: 基準日
//   - isReducedRate: 軽減税率フラグ（2019-10-01以降の飲食料品・新聞等）
//     WARNING: 対象が軽減税率の適用要件を満たすかの事実認定は呼び出し元の責任。
//
// エラー: 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合。
func CalcConsumptionTax(
	amount uint64,
	date time.Time,
	isReducedRate bool,
) (*ConsumptionTaxResult, error) {
	year := date.Year()
	month := int(date.Month())
	day := date.Day()
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

// StampTaxDocumentCode は印紙税の文書コードを表す。
type StampTaxDocumentCode string

const (
	StampTaxDocumentArticle1RealEstateTransfer      StampTaxDocumentCode = "article1_real_estate_transfer"
	StampTaxDocumentArticle1OtherTransfer           StampTaxDocumentCode = "article1_other_transfer"
	StampTaxDocumentArticle1LandLeaseOrSurfaceRight StampTaxDocumentCode = "article1_land_lease_or_surface_right"
	StampTaxDocumentArticle1ConsumptionLoan         StampTaxDocumentCode = "article1_consumption_loan"
	StampTaxDocumentArticle1Transportation          StampTaxDocumentCode = "article1_transportation"
	StampTaxDocumentArticle2ConstructionWork        StampTaxDocumentCode = "article2_construction_work"
	StampTaxDocumentArticle2GeneralContract         StampTaxDocumentCode = "article2_general_contract"
	StampTaxDocumentArticle3BillAmountTable         StampTaxDocumentCode = "article3_bill_amount_table"
	StampTaxDocumentArticle3BillSpecialFlat200      StampTaxDocumentCode = "article3_bill_special_flat_200"
	StampTaxDocumentArticle4SecurityCertificate     StampTaxDocumentCode = "article4_security_certificate"
	StampTaxDocumentArticle5MergerOrSplit           StampTaxDocumentCode = "article5_merger_or_split"
	StampTaxDocumentArticle6ArticlesOfIncorporation StampTaxDocumentCode = "article6_articles_of_incorporation"
	StampTaxDocumentArticle7ContinuingTransaction   StampTaxDocumentCode = "article7_continuing_transaction_basic"
	StampTaxDocumentArticle8DepositCertificate      StampTaxDocumentCode = "article8_deposit_certificate"
	StampTaxDocumentArticle9TransportCertificate    StampTaxDocumentCode = "article9_transport_certificate"
	StampTaxDocumentArticle10InsuranceCertificate   StampTaxDocumentCode = "article10_insurance_certificate"
	StampTaxDocumentArticle11LetterOfCredit         StampTaxDocumentCode = "article11_letter_of_credit"
	StampTaxDocumentArticle12TrustContract          StampTaxDocumentCode = "article12_trust_contract"
	StampTaxDocumentArticle13DebtGuarantee          StampTaxDocumentCode = "article13_debt_guarantee"
	StampTaxDocumentArticle14DepositContract        StampTaxDocumentCode = "article14_deposit_contract"
	StampTaxDocumentArticle15AssignmentOrAssumption StampTaxDocumentCode = "article15_assignment_or_assumption"
	StampTaxDocumentArticle16DividendReceipt        StampTaxDocumentCode = "article16_dividend_receipt"
	StampTaxDocumentArticle17SalesReceipt           StampTaxDocumentCode = "article17_sales_receipt"
	StampTaxDocumentArticle17OtherReceipt           StampTaxDocumentCode = "article17_other_receipt"
	StampTaxDocumentArticle18Passbook               StampTaxDocumentCode = "article18_passbook"
	StampTaxDocumentArticle19MiscPassbook           StampTaxDocumentCode = "article19_misc_passbook"
	StampTaxDocumentArticle20SealBook               StampTaxDocumentCode = "article20_seal_book"
)

// StampTaxFlag は印紙税の特例フラグを表す。
type StampTaxFlag string

const (
	StampTaxFlagArticle3CopyOrTranscriptExempt                 StampTaxFlag = "article3_copy_or_transcript_exempt"
	StampTaxFlagArticle4SpecifiedIssuerExempt                  StampTaxFlag = "article4_specified_issuer_exempt"
	StampTaxFlagArticle4RestrictedBeneficiaryCertificateExempt StampTaxFlag = "article4_restricted_beneficiary_certificate_exempt"
	StampTaxFlagArticle6NotaryCopyExempt                       StampTaxFlag = "article6_notary_copy_exempt"
	StampTaxFlagArticle8SmallDepositExempt                     StampTaxFlag = "article8_small_deposit_exempt"
	StampTaxFlagArticle13IdentityGuaranteeExempt               StampTaxFlag = "article13_identity_guarantee_exempt"
	StampTaxFlagArticle17NonBusinessExempt                     StampTaxFlag = "article17_non_business_exempt"
	StampTaxFlagArticle17AppendedReceiptExempt                 StampTaxFlag = "article17_appended_receipt_exempt"
	StampTaxFlagArticle18SpecifiedFinancialInstitutionExempt   StampTaxFlag = "article18_specified_financial_institution_exempt"
	StampTaxFlagArticle18IncomeTaxExemptPassbook               StampTaxFlag = "article18_income_tax_exempt_passbook"
	StampTaxFlagArticle18TaxReserveDepositPassbook             StampTaxFlag = "article18_tax_reserve_deposit_passbook"
)

// StampTaxResult は印紙税の計算結果を表す。
type StampTaxResult struct {
	// TaxAmount は印紙税額（円）。
	TaxAmount uint64
	// RuleLabel は適用された税額ルールの表示名。
	RuleLabel string
	// AppliedSpecialRule は適用された特例ルールコード。未適用時は nil。
	AppliedSpecialRule *string
}

// ─── 印紙税 Go 公開関数 ─────────────────────────────────────────────────────────

// CalcStampTax は印紙税法 別表第一に基づく印紙税額を計算する。
func CalcStampTax(
	documentCode StampTaxDocumentCode,
	statedAmount *uint64,
	date time.Time,
	flags []StampTaxFlag,
) (*StampTaxResult, error) {
	year := date.Year()
	month := int(date.Month())
	day := date.Day()
	var cResult C.JLawStampTaxResult
	errorBuf := (*C.char)(C.malloc(C.J_LAW_ERROR_BUF_LEN))
	defer C.free(unsafe.Pointer(errorBuf))

	documentCodeValue, err := stampTaxDocumentCodeToC(documentCode)
	if err != nil {
		return nil, err
	}
	flagsValue, err := stampTaxFlagsToC(flags)
	if err != nil {
		return nil, err
	}
	statedAmountValue := C.uint64_t(0)
	hasStatedAmount := C.int(0)
	if statedAmount != nil {
		statedAmountValue = C.uint64_t(*statedAmount)
		hasStatedAmount = 1
	}

	ret := C.j_law_calc_stamp_tax(
		documentCodeValue,
		statedAmountValue,
		hasStatedAmount,
		C.uint16_t(year),
		C.uint8_t(month),
		C.uint8_t(day),
		flagsValue,
		&cResult,
		errorBuf,
		C.J_LAW_ERROR_BUF_LEN,
	)
	if ret != 0 {
		return nil, errors.New(C.GoString(errorBuf))
	}

	var appliedSpecialRule *string
	if value := C.GoString(&cResult.applied_special_rule[0]); value != "" {
		appliedSpecialRule = &value
	}

	return &StampTaxResult{
		TaxAmount:          uint64(cResult.tax_amount),
		RuleLabel:          C.GoString(&cResult.rule_label[0]),
		AppliedSpecialRule: appliedSpecialRule,
	}, nil
}

func stampTaxDocumentCodeToC(documentCode StampTaxDocumentCode) (C.uint32_t, error) {
	switch documentCode {
	case StampTaxDocumentArticle1RealEstateTransfer:
		return 1, nil
	case StampTaxDocumentArticle1OtherTransfer:
		return 2, nil
	case StampTaxDocumentArticle1LandLeaseOrSurfaceRight:
		return 3, nil
	case StampTaxDocumentArticle1ConsumptionLoan:
		return 4, nil
	case StampTaxDocumentArticle1Transportation:
		return 5, nil
	case StampTaxDocumentArticle2ConstructionWork:
		return 6, nil
	case StampTaxDocumentArticle2GeneralContract:
		return 7, nil
	case StampTaxDocumentArticle3BillAmountTable:
		return 8, nil
	case StampTaxDocumentArticle3BillSpecialFlat200:
		return 9, nil
	case StampTaxDocumentArticle4SecurityCertificate:
		return 10, nil
	case StampTaxDocumentArticle5MergerOrSplit:
		return 11, nil
	case StampTaxDocumentArticle6ArticlesOfIncorporation:
		return 12, nil
	case StampTaxDocumentArticle7ContinuingTransaction:
		return 13, nil
	case StampTaxDocumentArticle8DepositCertificate:
		return 14, nil
	case StampTaxDocumentArticle9TransportCertificate:
		return 15, nil
	case StampTaxDocumentArticle10InsuranceCertificate:
		return 16, nil
	case StampTaxDocumentArticle11LetterOfCredit:
		return 17, nil
	case StampTaxDocumentArticle12TrustContract:
		return 18, nil
	case StampTaxDocumentArticle13DebtGuarantee:
		return 19, nil
	case StampTaxDocumentArticle14DepositContract:
		return 20, nil
	case StampTaxDocumentArticle15AssignmentOrAssumption:
		return 21, nil
	case StampTaxDocumentArticle16DividendReceipt:
		return 22, nil
	case StampTaxDocumentArticle17SalesReceipt:
		return 23, nil
	case StampTaxDocumentArticle17OtherReceipt:
		return 24, nil
	case StampTaxDocumentArticle18Passbook:
		return 25, nil
	case StampTaxDocumentArticle19MiscPassbook:
		return 26, nil
	case StampTaxDocumentArticle20SealBook:
		return 27, nil
	default:
		return 0, fmt.Errorf("unsupported stamp tax document code: %s", documentCode)
	}
}

func stampTaxFlagsToC(flags []StampTaxFlag) (C.uint64_t, error) {
	var bitset uint64
	for _, flag := range flags {
		switch flag {
		case StampTaxFlagArticle3CopyOrTranscriptExempt:
			bitset |= 1 << 0
		case StampTaxFlagArticle4SpecifiedIssuerExempt:
			bitset |= 1 << 1
		case StampTaxFlagArticle4RestrictedBeneficiaryCertificateExempt:
			bitset |= 1 << 2
		case StampTaxFlagArticle6NotaryCopyExempt:
			bitset |= 1 << 3
		case StampTaxFlagArticle8SmallDepositExempt:
			bitset |= 1 << 4
		case StampTaxFlagArticle13IdentityGuaranteeExempt:
			bitset |= 1 << 5
		case StampTaxFlagArticle17NonBusinessExempt:
			bitset |= 1 << 6
		case StampTaxFlagArticle17AppendedReceiptExempt:
			bitset |= 1 << 7
		case StampTaxFlagArticle18SpecifiedFinancialInstitutionExempt:
			bitset |= 1 << 8
		case StampTaxFlagArticle18IncomeTaxExemptPassbook:
			bitset |= 1 << 9
		case StampTaxFlagArticle18TaxReserveDepositPassbook:
			bitset |= 1 << 10
		default:
			return 0, fmt.Errorf("unsupported stamp tax flag: %s", flag)
		}
	}
	return C.uint64_t(bitset), nil
}
