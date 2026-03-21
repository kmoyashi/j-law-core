use std::collections::HashSet;
use std::str::FromStr;

use js_sys::{Array, Number};
use wasm_bindgen::{prelude::*, JsCast};

use ::j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use ::j_law_core::domains::income_tax::{
    assessment::calculate_income_tax_assessment as calculate_income_tax_assessment_core,
    assessment::IncomeTaxAssessmentContext,
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    deduction::calculate_income_deductions as calculate_income_deductions_core,
    deduction::{
        DependentDeductionInput, DonationDeductionInput, ExpenseDeductionInput,
        IncomeDeductionContext, IncomeDeductionInput, IncomeDeductionKind,
        LifeInsuranceDeductionInput, MedicalDeductionInput, PersonalDeductionInput,
        SpouseDeductionInput,
    },
    policy::StandardIncomeTaxPolicy,
};
use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMlitPolicy,
    RealEstateFlag,
};
use ::j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxDocumentCode, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use ::j_law_core::domains::withholding_tax::{
    calculator::calculate_withholding_tax,
    context::{WithholdingTaxContext, WithholdingTaxFlag},
    policy::StandardWithholdingTaxPolicy,
    WithholdingTaxCategory,
};
use ::j_law_core::{InputError, JLawError, LegalDate};
use ::j_law_registry::load_brokerage_fee_params;
use ::j_law_registry::load_consumption_tax_params;
use ::j_law_registry::load_income_tax_deduction_params;
use ::j_law_registry::load_income_tax_params;
use ::j_law_registry::load_stamp_tax_params;
use ::j_law_registry::load_withholding_tax_params;

// ─── 日付ユーティリティ ──────────────────────────────────────────────────────────

/// JavaScript Date オブジェクトから JST（UTC+9）で解釈した (year, month, day) を返す。
///
/// `Date` オブジェクトのローカル時刻はブラウザ/Node.js の実行環境に依存するため、
/// タイムスタンプに JST オフセット（+9h）を加算して UTC 成分として読み出すことで
/// 実行環境のタイムゾーンに関わらず常に JST での日付を返す。
fn extract_jst_date(date: &js_sys::Date) -> (u16, u8, u8) {
    const JST_OFFSET_MS: f64 = 9.0 * 60.0 * 60.0 * 1000.0;
    let jst = js_sys::Date::new(&JsValue::from_f64(date.get_time() + JST_OFFSET_MS));
    let year = jst.get_utc_full_year() as u16;
    let month = (jst.get_utc_month() + 1) as u8;
    let day = jst.get_utc_date() as u8;
    (year, month, day)
}

fn get_prop_any(obj: &JsValue, keys: &[&str]) -> Result<JsValue, JsValue> {
    for key in keys {
        let value = js_sys::Reflect::get(obj, &JsValue::from_str(key))?;
        if !value.is_undefined() {
            return Ok(value);
        }
    }
    Ok(JsValue::UNDEFINED)
}

fn get_required_u64(obj: &JsValue, keys: &[&str]) -> Result<u64, JsValue> {
    let value = get_prop_any(obj, keys)?;
    if value.is_undefined() || value.is_null() {
        Err(JsValue::from_str(&format!(
            "missing numeric field: {}",
            keys[0]
        )))
    } else {
        parse_js_u64(&value, keys[0])
    }
}

fn get_optional_u64(obj: &JsValue, keys: &[&str], default: u64) -> Result<u64, JsValue> {
    let value = get_prop_any(obj, keys)?;
    if value.is_undefined() || value.is_null() {
        Ok(default)
    } else {
        parse_js_u64(&value, keys[0])
    }
}

fn invalid_deduction_input(field: &str, reason: &str) -> JsValue {
    JsValue::from_str(
        &JLawError::from(InputError::InvalidDeductionInput {
            field: field.to_string(),
            reason: reason.to_string(),
        })
        .to_string(),
    )
}

fn bigint_js_value(value: u64) -> JsValue {
    js_sys::BigInt::from(value).into()
}

/// f64 引数として渡された金額（円）を u64 に変換する。
///
/// - 非負の安全整数（`Number.isSafeInteger` かつ >= 0）でない場合はエラーを返す。
/// - JavaScript の `Number.MAX_SAFE_INTEGER`（2^53 - 1）を超える値はエラー。
fn f64_to_u64_checked(value: f64, field: &str) -> Result<u64, JsValue> {
    let js_val = JsValue::from_f64(value);
    if !Number::is_safe_integer(&js_val) || value < 0.0 {
        return Err(JsValue::from_str(&format!(
            "{field}: must be a non-negative safe integer (got {value})"
        )));
    }
    Ok(value as u64)
}

/// u64 の円金額を u32 に変換する。u32::MAX（約 42.9 億円）を超える場合はエラーを返す。
fn yen_u64_to_u32(value: u64, field: &str) -> Result<u32, JsValue> {
    u32::try_from(value).map_err(|_| {
        JsValue::from_str(&format!(
            "{field}: amount {value} yen exceeds u32 maximum ({max})",
            max = u32::MAX
        ))
    })
}

/// Optional<f64> の金額を Optional<u64> に変換する。
fn opt_f64_to_u64_checked(value: Option<f64>, field: &str) -> Result<Option<u64>, JsValue> {
    match value {
        None => Ok(None),
        Some(v) => f64_to_u64_checked(v, field).map(Some),
    }
}

fn parse_js_u64(value: &JsValue, field: &str) -> Result<u64, JsValue> {
    if value.is_bigint() {
        let bigint = value.clone().dyn_into::<js_sys::BigInt>().map_err(|_| {
            invalid_deduction_input(
                field,
                "must be a non-negative safe integer Number or BigInt",
            )
        })?;
        return u64::try_from(bigint).map_err(|_| {
            invalid_deduction_input(
                field,
                "must be a non-negative safe integer Number or BigInt",
            )
        });
    }

    let Some(number) = value.as_f64() else {
        return Err(invalid_deduction_input(
            field,
            "must be a non-negative safe integer Number or BigInt",
        ));
    };

    if !Number::is_safe_integer(value) || number < 0.0 {
        return Err(invalid_deduction_input(
            field,
            "must be a non-negative safe integer Number or BigInt",
        ));
    }

    Ok(number as u64)
}

fn get_optional_u16(obj: &JsValue, keys: &[&str], default: u16) -> Result<u16, JsValue> {
    let value = get_optional_u64(obj, keys, u64::from(default))?;
    u16::try_from(value)
        .map_err(|_| invalid_deduction_input(keys[0], &format!("count must be <= {}", u16::MAX)))
}

fn get_required_bool(obj: &JsValue, keys: &[&str]) -> Result<bool, JsValue> {
    let value = get_prop_any(obj, keys)?;
    value
        .as_bool()
        .ok_or_else(|| JsValue::from_str(&format!("missing boolean field: {}", keys[0])))
}

fn get_optional_bool(obj: &JsValue, keys: &[&str], default: bool) -> Result<bool, JsValue> {
    let value = get_prop_any(obj, keys)?;
    if value.is_undefined() || value.is_null() {
        Ok(default)
    } else {
        value
            .as_bool()
            .ok_or_else(|| JsValue::from_str(&format!("invalid boolean field: {}", keys[0])))
    }
}

fn get_optional_object(obj: &JsValue, keys: &[&str]) -> Result<Option<JsValue>, JsValue> {
    let value = get_prop_any(obj, keys)?;
    if value.is_undefined() || value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn get_required_date(obj: &JsValue, keys: &[&str]) -> Result<js_sys::Date, JsValue> {
    let value = get_prop_any(obj, keys)?;
    if value.is_instance_of::<js_sys::Date>() {
        value.dyn_into::<js_sys::Date>()
    } else {
        Err(JsValue::from_str(&format!(
            "missing Date field: {}",
            keys[0]
        )))
    }
}

fn to_income_deduction_context(input: &JsValue) -> Result<IncomeDeductionContext, JsValue> {
    let total_income_amount =
        get_required_u64(input, &["totalIncomeAmount", "total_income_amount"])?;
    let date = get_required_date(input, &["date"])?;
    let (year, month, day) = extract_jst_date(&date);

    let spouse = match get_optional_object(input, &["spouse"])? {
        Some(spouse) => Some(SpouseDeductionInput {
            spouse_total_income_amount: get_required_u64(
                &spouse,
                &["spouseTotalIncomeAmount", "spouse_total_income_amount"],
            )?,
            is_same_household: get_required_bool(
                &spouse,
                &["isSameHousehold", "is_same_household"],
            )?,
            is_elderly: get_optional_bool(&spouse, &["isElderly", "is_elderly"], false)?,
        }),
        None => None,
    };
    let dependent = match get_optional_object(input, &["dependent"])? {
        Some(dependent) => DependentDeductionInput {
            general_count: get_optional_u16(&dependent, &["generalCount", "general_count"], 0)?,
            specific_count: get_optional_u16(&dependent, &["specificCount", "specific_count"], 0)?,
            elderly_cohabiting_count: get_optional_u16(
                &dependent,
                &["elderlyCohabitingCount", "elderly_cohabiting_count"],
                0,
            )?,
            elderly_other_count: get_optional_u16(
                &dependent,
                &["elderlyOtherCount", "elderly_other_count"],
                0,
            )?,
        },
        None => DependentDeductionInput::default(),
    };
    let medical = match get_optional_object(input, &["medical"])? {
        Some(medical) => Some(MedicalDeductionInput {
            medical_expense_paid: get_required_u64(
                &medical,
                &["medicalExpensePaid", "medical_expense_paid"],
            )?,
            reimbursed_amount: get_optional_u64(
                &medical,
                &["reimbursedAmount", "reimbursed_amount"],
                0,
            )?,
        }),
        None => None,
    };
    let life_insurance = match get_optional_object(input, &["lifeInsurance", "life_insurance"])? {
        Some(life) => Some(LifeInsuranceDeductionInput {
            new_general_paid_amount: get_optional_u64(
                &life,
                &["newGeneralPaidAmount", "new_general_paid_amount"],
                0,
            )?,
            new_individual_pension_paid_amount: get_optional_u64(
                &life,
                &[
                    "newIndividualPensionPaidAmount",
                    "new_individual_pension_paid_amount",
                ],
                0,
            )?,
            new_care_medical_paid_amount: get_optional_u64(
                &life,
                &["newCareMedicalPaidAmount", "new_care_medical_paid_amount"],
                0,
            )?,
            old_general_paid_amount: get_optional_u64(
                &life,
                &["oldGeneralPaidAmount", "old_general_paid_amount"],
                0,
            )?,
            old_individual_pension_paid_amount: get_optional_u64(
                &life,
                &[
                    "oldIndividualPensionPaidAmount",
                    "old_individual_pension_paid_amount",
                ],
                0,
            )?,
        }),
        None => None,
    };
    let donation = match get_optional_object(input, &["donation"])? {
        Some(donation) => Some(DonationDeductionInput {
            qualified_donation_amount: get_required_u64(
                &donation,
                &["qualifiedDonationAmount", "qualified_donation_amount"],
            )?,
        }),
        None => None,
    };

    Ok(IncomeDeductionContext {
        total_income_amount,
        target_date: LegalDate::new(year, month, day),
        deductions: IncomeDeductionInput {
            personal: PersonalDeductionInput { spouse, dependent },
            expense: ExpenseDeductionInput {
                social_insurance_premium_paid: get_optional_u64(
                    input,
                    &[
                        "socialInsurancePremiumPaid",
                        "social_insurance_premium_paid",
                    ],
                    0,
                )?,
                medical,
                life_insurance,
                donation,
            },
        },
    })
}

fn income_deduction_kind_to_js(kind: IncomeDeductionKind) -> u32 {
    match kind {
        IncomeDeductionKind::Basic => 1,
        IncomeDeductionKind::Spouse => 2,
        IncomeDeductionKind::Dependent => 3,
        IncomeDeductionKind::SocialInsurance => 4,
        IncomeDeductionKind::Medical => 5,
        IncomeDeductionKind::LifeInsurance => 6,
        IncomeDeductionKind::Donation => 7,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  消費税
// ═══════════════════════════════════════════════════════════════════════════════

/// 消費税の計算結果。
///
/// Properties:
/// - `taxAmount`: 消費税額（円, JavaScript では BigInt）
/// - `amountWithTax`: 税込金額（円, JavaScript では BigInt）
/// - `amountWithoutTax`: 税抜金額（円, JavaScript では BigInt）
/// - `appliedRateNumer`: 適用税率の分子
/// - `appliedRateDenom`: 適用税率の分母
/// - `isReducedRate`: 軽減税率が適用されたか
#[wasm_bindgen]
pub struct ConsumptionTaxResult {
    tax_amount: u64,
    amount_with_tax: u64,
    amount_without_tax: u64,
    applied_rate_numer: u32,
    applied_rate_denom: u32,
    is_reduced_rate: bool,
}

#[wasm_bindgen]
impl ConsumptionTaxResult {
    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    #[wasm_bindgen(getter, js_name = "amountWithTax")]
    pub fn amount_with_tax(&self) -> u64 {
        self.amount_with_tax
    }

    #[wasm_bindgen(getter, js_name = "amountWithoutTax")]
    pub fn amount_without_tax(&self) -> u64 {
        self.amount_without_tax
    }

    #[wasm_bindgen(getter, js_name = "appliedRateNumer")]
    pub fn applied_rate_numer(&self) -> u32 {
        self.applied_rate_numer
    }

    #[wasm_bindgen(getter, js_name = "appliedRateDenom")]
    pub fn applied_rate_denom(&self) -> u32 {
        self.applied_rate_denom
    }

    #[wasm_bindgen(getter, js_name = "isReducedRate")]
    pub fn is_reduced_rate(&self) -> bool {
        self.is_reduced_rate
    }
}

/// 消費税法第29条に基づく消費税額を計算する。
///
/// @param amount - 課税標準額（税抜き・円）。JavaScript の Number 型は 53bit 整数精度のため
///   u64 を直接受け取れないため、安全な整数 Number のみ受け付ける。
/// @param date - 基準日（JavaScript Date オブジェクト。JST で解釈される）
/// @param isReducedRate - 軽減税率フラグ（2019-10-01以降の飲食料品・新聞等）
/// @returns ConsumptionTaxResult
/// @throws 入力値が安全な整数でない場合
/// @throws 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合
#[wasm_bindgen(js_name = "calcConsumptionTax")]
pub fn calc_consumption_tax(
    amount: f64,
    date: &js_sys::Date,
    is_reduced_rate: bool,
) -> Result<ConsumptionTaxResult, JsValue> {
    let (year, month, day) = extract_jst_date(date);
    let target_date = LegalDate::new(year, month, day);

    let params =
        load_consumption_tax_params(target_date).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if is_reduced_rate {
        flags.insert(ConsumptionTaxFlag::ReducedRate);
    }

    let ctx = ConsumptionTaxContext {
        amount: f64_to_u64_checked(amount, "amount")?,
        target_date,
        flags,
        policy: Box::new(StandardConsumptionTaxPolicy),
    };

    let result =
        calculate_consumption_tax(&ctx, &params).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(ConsumptionTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        amount_with_tax: result.amount_with_tax.as_yen(),
        amount_without_tax: result.amount_without_tax.as_yen(),
        applied_rate_numer: u32::try_from(result.applied_rate_numer)
            .map_err(|_| JsValue::from_str("applied_rate_numer overflows u32"))?,
        applied_rate_denom: u32::try_from(result.applied_rate_denom)
            .map_err(|_| JsValue::from_str("applied_rate_denom overflows u32"))?,
        is_reduced_rate: result.is_reduced_rate,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  不動産（宅建業法）
// ═══════════════════════════════════════════════════════════════════════════════

/// 媒介報酬の計算結果。
///
/// Properties:
/// - `totalWithoutTax`: 税抜合計額（円）BigInt
/// - `totalWithTax`: 税込合計額（円）BigInt
/// - `taxAmount`: 消費税額（円）BigInt
/// - `lowCostSpecialApplied`: 低廉な空き家特例が適用されたか
/// - `breakdown()`: 各ティアの計算内訳（`{ label, baseAmount, rateNumer, rateDenom, result }[]`）
#[wasm_bindgen]
pub struct BrokerageFeeResult {
    total_without_tax: u64,
    total_with_tax: u64,
    tax_amount: u64,
    low_cost_special_applied: bool,
    breakdown_data: Vec<BreakdownStepData>,
}

struct BreakdownStepData {
    label: String,
    base_amount: u64,
    rate_numer: u32,
    rate_denom: u32,
    result: u64,
}

#[wasm_bindgen]
impl BrokerageFeeResult {
    #[wasm_bindgen(getter, js_name = "totalWithoutTax")]
    pub fn total_without_tax(&self) -> u64 {
        self.total_without_tax
    }

    #[wasm_bindgen(getter, js_name = "totalWithTax")]
    pub fn total_with_tax(&self) -> u64 {
        self.total_with_tax
    }

    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    #[wasm_bindgen(getter, js_name = "lowCostSpecialApplied")]
    pub fn low_cost_special_applied(&self) -> bool {
        self.low_cost_special_applied
    }

    pub fn breakdown(&self) -> Array {
        self.breakdown_data
            .iter()
            .map(|step| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&step.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("baseAmount"),
                    &bigint_js_value(step.base_amount),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateNumer"),
                    &JsValue::from_f64(step.rate_numer as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateDenom"),
                    &JsValue::from_f64(step.rate_denom as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("result"),
                    &bigint_js_value(step.result),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

#[wasm_bindgen(js_name = "calcBrokerageFee")]
pub fn calc_brokerage_fee(
    price: f64,
    date: &js_sys::Date,
    is_low_cost_vacant_house: bool,
    is_seller: bool,
) -> Result<BrokerageFeeResult, JsValue> {
    let price = f64_to_u64_checked(price, "price")?;
    let (year, month, day) = extract_jst_date(date);

    let params = load_brokerage_fee_params(LegalDate::new(year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if is_low_cost_vacant_house {
        flags.insert(RealEstateFlag::IsLowCostVacantHouse);
    }
    if is_seller {
        flags.insert(RealEstateFlag::IsSeller);
    }

    let ctx = RealEstateContext {
        price,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardMlitPolicy),
    };

    let result =
        calculate_brokerage_fee(&ctx, &params).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|step| {
            Ok(BreakdownStepData {
                label: step.label.clone(),
                base_amount: step.base_amount,
                rate_numer: u32::try_from(step.rate_numer)
                    .map_err(|_| JsValue::from_str("breakdown.rate_numer overflows u32"))?,
                rate_denom: u32::try_from(step.rate_denom)
                    .map_err(|_| JsValue::from_str("breakdown.rate_denom overflows u32"))?,
                result: step.result.as_yen(),
            })
        })
        .collect::<Result<Vec<_>, JsValue>>()?;

    Ok(BrokerageFeeResult {
        total_without_tax: result.total_without_tax.as_yen(),
        total_with_tax: result.total_with_tax.as_yen(),
        tax_amount: result.tax_amount.as_yen(),
        low_cost_special_applied: result.low_cost_special_applied,
        breakdown_data,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  所得税
// ═══════════════════════════════════════════════════════════════════════════════

/// 所得税の計算結果。
///
/// Properties:
/// - `baseTax`: 基準所得税額（円）BigInt
/// - `reconstructionTax`: 復興特別所得税額（円）BigInt
/// - `totalTax`: 申告納税額（円・100円未満切り捨て）BigInt
/// - `reconstructionTaxApplied`: 復興特別所得税が適用されたか
/// - `breakdown()`: 計算内訳（`{ label, taxableIncome, rateNumer, rateDenom, deduction, result }[]`）
#[wasm_bindgen]
pub struct IncomeTaxResult {
    base_tax: u64,
    reconstruction_tax: u64,
    total_tax: u64,
    reconstruction_tax_applied: bool,
    breakdown_data: Vec<IncomeTaxStepData>,
}

struct IncomeTaxStepData {
    label: String,
    taxable_income: u64,
    rate_numer: u32,
    rate_denom: u32,
    deduction: u64,
    result: u64,
}

#[wasm_bindgen]
impl IncomeTaxResult {
    #[wasm_bindgen(getter, js_name = "baseTax")]
    pub fn base_tax(&self) -> u64 {
        self.base_tax
    }

    #[wasm_bindgen(getter, js_name = "reconstructionTax")]
    pub fn reconstruction_tax(&self) -> u64 {
        self.reconstruction_tax
    }

    #[wasm_bindgen(getter, js_name = "totalTax")]
    pub fn total_tax(&self) -> u64 {
        self.total_tax
    }

    #[wasm_bindgen(getter, js_name = "reconstructionTaxApplied")]
    pub fn reconstruction_tax_applied(&self) -> bool {
        self.reconstruction_tax_applied
    }

    pub fn breakdown(&self) -> Array {
        self.breakdown_data
            .iter()
            .map(|step| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&step.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("taxableIncome"),
                    &bigint_js_value(step.taxable_income),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateNumer"),
                    &JsValue::from_f64(step.rate_numer as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateDenom"),
                    &JsValue::from_f64(step.rate_denom as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("deduction"),
                    &bigint_js_value(step.deduction),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("result"),
                    &bigint_js_value(step.result),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// @param taxableIncome - 課税所得金額（円）。JavaScript の Number 型は 53bit 整数精度のため
///   安全な整数 Number のみ受け付ける。
/// @param date - 基準日（JavaScript Date オブジェクト。JST で解釈される）
/// @param applyReconstructionTax - 復興特別所得税を適用するか
/// @returns IncomeTaxResult（金額フィールドは BigInt）
/// @throws 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合
#[wasm_bindgen(js_name = "calcIncomeTax")]
pub fn calc_income_tax(
    taxable_income: f64,
    date: &js_sys::Date,
    apply_reconstruction_tax: bool,
) -> Result<IncomeTaxResult, JsValue> {
    let taxable_income = f64_to_u64_checked(taxable_income, "taxableIncome")?;
    let (year, month, day) = extract_jst_date(date);

    let params = load_income_tax_params(LegalDate::new(year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if apply_reconstruction_tax {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }

    let ctx = IncomeTaxContext {
        taxable_income,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let result =
        calculate_income_tax(&ctx, &params).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|step| {
            Ok(IncomeTaxStepData {
                label: step.label.clone(),
                taxable_income: step.taxable_income,
                rate_numer: u32::try_from(step.rate_numer)
                    .map_err(|_| JsValue::from_str("breakdown.rate_numer overflows u32"))?,
                rate_denom: u32::try_from(step.rate_denom)
                    .map_err(|_| JsValue::from_str("breakdown.rate_denom overflows u32"))?,
                deduction: step.deduction,
                result: step.result.as_yen(),
            })
        })
        .collect::<Result<Vec<_>, JsValue>>()?;

    Ok(IncomeTaxResult {
        base_tax: result.base_tax.as_yen(),
        reconstruction_tax: result.reconstruction_tax.as_yen(),
        total_tax: result.total_tax.as_yen(),
        reconstruction_tax_applied: result.reconstruction_tax_applied,
        breakdown_data,
    })
}

/// 所得控除の計算結果。
#[wasm_bindgen]
pub struct IncomeDeductionResult {
    total_income_amount: u64,
    total_deductions: u64,
    taxable_income_before_truncation: u64,
    taxable_income: u64,
    breakdown_data: Vec<IncomeDeductionLineData>,
}

struct IncomeDeductionLineData {
    kind: u32,
    label: String,
    amount: u64,
}

#[wasm_bindgen]
impl IncomeDeductionResult {
    #[wasm_bindgen(getter, js_name = "totalIncomeAmount")]
    pub fn total_income_amount(&self) -> u64 {
        self.total_income_amount
    }

    #[wasm_bindgen(getter, js_name = "totalDeductions")]
    pub fn total_deductions(&self) -> u64 {
        self.total_deductions
    }

    #[wasm_bindgen(getter, js_name = "taxableIncomeBeforeTruncation")]
    pub fn taxable_income_before_truncation(&self) -> u64 {
        self.taxable_income_before_truncation
    }

    #[wasm_bindgen(getter, js_name = "taxableIncome")]
    pub fn taxable_income(&self) -> u64 {
        self.taxable_income
    }

    pub fn breakdown(&self) -> Array {
        self.breakdown_data
            .iter()
            .map(|line| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("kind"),
                    &JsValue::from_f64(line.kind as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&line.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("amount"),
                    &bigint_js_value(line.amount),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

struct IncomeTaxAssessmentStepData {
    label: String,
    taxable_income: u64,
    rate_numer: u32,
    rate_denom: u32,
    deduction: u64,
    result: u64,
}

/// 所得控除から所得税額までの通し計算結果。
#[wasm_bindgen]
pub struct IncomeTaxAssessmentResult {
    total_income_amount: u64,
    total_deductions: u64,
    taxable_income_before_truncation: u64,
    taxable_income: u64,
    base_tax: u64,
    reconstruction_tax: u64,
    total_tax: u64,
    reconstruction_tax_applied: bool,
    deduction_breakdown_data: Vec<IncomeDeductionLineData>,
    tax_breakdown_data: Vec<IncomeTaxAssessmentStepData>,
}

#[wasm_bindgen]
impl IncomeTaxAssessmentResult {
    #[wasm_bindgen(getter, js_name = "totalIncomeAmount")]
    pub fn total_income_amount(&self) -> u64 {
        self.total_income_amount
    }

    #[wasm_bindgen(getter, js_name = "totalDeductions")]
    pub fn total_deductions(&self) -> u64 {
        self.total_deductions
    }

    #[wasm_bindgen(getter, js_name = "taxableIncomeBeforeTruncation")]
    pub fn taxable_income_before_truncation(&self) -> u64 {
        self.taxable_income_before_truncation
    }

    #[wasm_bindgen(getter, js_name = "taxableIncome")]
    pub fn taxable_income(&self) -> u64 {
        self.taxable_income
    }

    #[wasm_bindgen(getter, js_name = "baseTax")]
    pub fn base_tax(&self) -> u64 {
        self.base_tax
    }

    #[wasm_bindgen(getter, js_name = "reconstructionTax")]
    pub fn reconstruction_tax(&self) -> u64 {
        self.reconstruction_tax
    }

    #[wasm_bindgen(getter, js_name = "totalTax")]
    pub fn total_tax(&self) -> u64 {
        self.total_tax
    }

    #[wasm_bindgen(getter, js_name = "reconstructionTaxApplied")]
    pub fn reconstruction_tax_applied(&self) -> bool {
        self.reconstruction_tax_applied
    }

    #[wasm_bindgen(js_name = "deductionBreakdown")]
    pub fn deduction_breakdown(&self) -> Array {
        self.deduction_breakdown_data
            .iter()
            .map(|line| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("kind"),
                    &JsValue::from_f64(line.kind as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&line.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("amount"),
                    &bigint_js_value(line.amount),
                );
                JsValue::from(obj)
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "taxBreakdown")]
    pub fn tax_breakdown(&self) -> Array {
        self.tax_breakdown_data
            .iter()
            .map(|step| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&step.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("taxableIncome"),
                    &bigint_js_value(step.taxable_income),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateNumer"),
                    &JsValue::from_f64(step.rate_numer as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateDenom"),
                    &JsValue::from_f64(step.rate_denom as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("deduction"),
                    &bigint_js_value(step.deduction),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("result"),
                    &bigint_js_value(step.result),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

#[wasm_bindgen(js_name = "calcIncomeDeductions")]
pub fn calc_income_deductions(input: &JsValue) -> Result<IncomeDeductionResult, JsValue> {
    let deduction_context = to_income_deduction_context(input)?;
    let params = load_income_tax_deduction_params(deduction_context.target_date)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = calculate_income_deductions_core(&deduction_context, &params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|line| IncomeDeductionLineData {
            kind: income_deduction_kind_to_js(line.kind),
            label: line.label.clone(),
            amount: line.amount.as_yen(),
        })
        .collect();

    Ok(IncomeDeductionResult {
        total_income_amount: result.total_income_amount.as_yen(),
        total_deductions: result.total_deductions.as_yen(),
        taxable_income_before_truncation: result.taxable_income_before_truncation.as_yen(),
        taxable_income: result.taxable_income.as_yen(),
        breakdown_data,
    })
}

#[wasm_bindgen(js_name = "calcIncomeTaxAssessment")]
pub fn calc_income_tax_assessment(
    input: &JsValue,
    apply_reconstruction_tax: bool,
) -> Result<IncomeTaxAssessmentResult, JsValue> {
    let deduction_context = to_income_deduction_context(input)?;
    let deduction_params = load_income_tax_deduction_params(deduction_context.target_date)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let tax_params = load_income_tax_params(deduction_context.target_date)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if apply_reconstruction_tax {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }
    let ctx = IncomeTaxAssessmentContext {
        deduction_context,
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };
    let result = calculate_income_tax_assessment_core(&ctx, &deduction_params, &tax_params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let deduction_breakdown_data = result
        .deductions
        .breakdown
        .iter()
        .map(|line| IncomeDeductionLineData {
            kind: income_deduction_kind_to_js(line.kind),
            label: line.label.clone(),
            amount: line.amount.as_yen(),
        })
        .collect();
    let tax_breakdown_data = result
        .tax
        .breakdown
        .iter()
        .map(|step| {
            Ok(IncomeTaxAssessmentStepData {
                label: step.label.clone(),
                taxable_income: step.taxable_income,
                rate_numer: u32::try_from(step.rate_numer)
                    .map_err(|_| JsValue::from_str("tax_breakdown.rate_numer overflows u32"))?,
                rate_denom: u32::try_from(step.rate_denom)
                    .map_err(|_| JsValue::from_str("tax_breakdown.rate_denom overflows u32"))?,
                deduction: step.deduction,
                result: step.result.as_yen(),
            })
        })
        .collect::<Result<Vec<_>, JsValue>>()?;

    Ok(IncomeTaxAssessmentResult {
        total_income_amount: result.deductions.total_income_amount.as_yen(),
        total_deductions: result.deductions.total_deductions.as_yen(),
        taxable_income_before_truncation: result
            .deductions
            .taxable_income_before_truncation
            .as_yen(),
        taxable_income: result.deductions.taxable_income.as_yen(),
        base_tax: result.tax.base_tax.as_yen(),
        reconstruction_tax: result.tax.reconstruction_tax.as_yen(),
        total_tax: result.tax.total_tax.as_yen(),
        reconstruction_tax_applied: result.tax.reconstruction_tax_applied,
        deduction_breakdown_data,
        tax_breakdown_data,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  印紙税
// ═══════════════════════════════════════════════════════════════════════════════

/// 印紙税の計算結果。
///
/// Properties:
/// - `taxAmount`: 印紙税額（円）
/// - `ruleLabel`: 適用された税額ルールの表示名
/// - `appliedSpecialRule`: 適用された特例ルールコード
#[wasm_bindgen]
pub struct StampTaxResult {
    tax_amount: u64,
    rule_label: String,
    applied_special_rule: Option<String>,
}

#[wasm_bindgen]
impl StampTaxResult {
    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    #[wasm_bindgen(getter, js_name = "ruleLabel")]
    pub fn rule_label(&self) -> String {
        self.rule_label.clone()
    }

    #[wasm_bindgen(getter, js_name = "appliedSpecialRule")]
    pub fn applied_special_rule(&self) -> Option<String> {
        self.applied_special_rule.clone()
    }
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// @param documentCode - 文書コード
/// @param statedAmount - 記載金額（円）。記載がない文書は `null`
/// @param date - 契約書作成日（JavaScript Date オブジェクト。JST で解釈される）
/// @param flags - 主な非課税文書フラグ配列
/// @returns StampTaxResult
/// @throws 入力が不正、または対象日に有効な法令パラメータが存在しない場合
/// JavaScript の Number 型は 53bit 整数精度のため u64 を直接受け取れない。
#[wasm_bindgen(js_name = "calcStampTax")]
pub fn calc_stamp_tax(
    document_code: String,
    stated_amount: Option<f64>,
    date: &js_sys::Date,
    flag_values: Option<Box<[JsValue]>>,
) -> Result<StampTaxResult, JsValue> {
    let (year, month, day) = extract_jst_date(date);
    let target_date = LegalDate::new(year, month, day);
    let document_code = document_code
        .parse::<StampTaxDocumentCode>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let stated_amount = opt_f64_to_u64_checked(stated_amount, "statedAmount")?;

    let params =
        load_stamp_tax_params(target_date).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if let Some(entries) = flag_values {
        for entry in entries.iter() {
            let flag = entry
                .as_string()
                .ok_or_else(|| JsValue::from_str("stamp tax flags must be strings"))?
                .parse::<StampTaxFlag>()
                .map_err(|e: InputError| JsValue::from_str(&e.to_string()))?;
            flags.insert(flag);
        }
    }

    let ctx = StampTaxContext {
        document_code,
        stated_amount,
        target_date,
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    let result =
        calculate_stamp_tax(&ctx, &params).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(StampTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        rule_label: result.rule_label,
        applied_special_rule: result.applied_special_rule,
    })
}

/// 源泉徴収税額の計算結果。
#[wasm_bindgen]
pub struct WithholdingTaxResult {
    gross_payment_amount: u64,
    taxable_payment_amount: u64,
    tax_amount: u64,
    net_payment_amount: u64,
    category: String,
    submission_prize_exempted: bool,
    breakdown_data: Vec<BreakdownStepData>,
}

#[wasm_bindgen]
impl WithholdingTaxResult {
    #[wasm_bindgen(getter, js_name = "grossPaymentAmount")]
    pub fn gross_payment_amount(&self) -> u64 {
        self.gross_payment_amount
    }

    #[wasm_bindgen(getter, js_name = "taxablePaymentAmount")]
    pub fn taxable_payment_amount(&self) -> u64 {
        self.taxable_payment_amount
    }

    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    #[wasm_bindgen(getter, js_name = "netPaymentAmount")]
    pub fn net_payment_amount(&self) -> u64 {
        self.net_payment_amount
    }

    #[wasm_bindgen(getter)]
    pub fn category(&self) -> String {
        self.category.clone()
    }

    #[wasm_bindgen(getter, js_name = "submissionPrizeExempted")]
    pub fn submission_prize_exempted(&self) -> bool {
        self.submission_prize_exempted
    }

    pub fn breakdown(&self) -> Array {
        self.breakdown_data
            .iter()
            .map(|step| {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&step.label),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("baseAmount"),
                    &bigint_js_value(step.base_amount),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateNumer"),
                    &JsValue::from_f64(step.rate_numer as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("rateDenom"),
                    &JsValue::from_f64(step.rate_denom as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("result"),
                    &bigint_js_value(step.result),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

/// 所得税法第204条第1項に基づく報酬・料金等の源泉徴収税額を計算する。
///
/// @param paymentAmount - 支払総額（円）。JavaScript の Number 型は 53bit 整数精度のため
///   安全な整数 Number のみ受け付ける。
/// @param date - 基準日（JavaScript Date オブジェクト。JST で解釈される）
/// @param category - `"manuscript_and_lecture" | "professional_fee" | "exclusive_contract_fee"`
/// @param isSubmissionPrize - 応募作品等の入選賞金・謝金として扱うか
/// @param separatedConsumptionTaxAmount - 区分表示された消費税額（円）
/// @returns WithholdingTaxResult（金額フィールドは BigInt）
#[wasm_bindgen(js_name = "calcWithholdingTax")]
pub fn calc_withholding_tax_js(
    payment_amount: f64,
    date: &js_sys::Date,
    category: &str,
    is_submission_prize: bool,
    separated_consumption_tax_amount: f64,
) -> Result<WithholdingTaxResult, JsValue> {
    let payment_amount = f64_to_u64_checked(payment_amount, "paymentAmount")?;
    let separated_consumption_tax_amount =
        f64_to_u64_checked(separated_consumption_tax_amount, "separatedConsumptionTaxAmount")?;
    let (year, month, day) = extract_jst_date(date);
    let params = load_withholding_tax_params(LegalDate::new(year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let category = WithholdingTaxCategory::from_str(category)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if is_submission_prize {
        flags.insert(WithholdingTaxFlag::IsSubmissionPrize);
    }

    let ctx = WithholdingTaxContext {
        payment_amount,
        separated_consumption_tax_amount,
        category,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardWithholdingTaxPolicy),
    };

    let result =
        calculate_withholding_tax(&ctx, &params).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|step| {
            Ok(BreakdownStepData {
                label: step.label.clone(),
                base_amount: step.base_amount,
                rate_numer: u32::try_from(step.rate_numer)
                    .map_err(|_| JsValue::from_str("breakdown.rate_numer overflows u32"))?,
                rate_denom: u32::try_from(step.rate_denom)
                    .map_err(|_| JsValue::from_str("breakdown.rate_denom overflows u32"))?,
                result: step.result.as_yen(),
            })
        })
        .collect::<Result<Vec<_>, JsValue>>()?;

    Ok(WithholdingTaxResult {
        gross_payment_amount: result.gross_payment_amount.as_yen(),
        taxable_payment_amount: result.taxable_payment_amount.as_yen(),
        tax_amount: result.tax_amount.as_yen(),
        net_payment_amount: result.net_payment_amount.as_yen(),
        category: result.category.to_string(),
        submission_prize_exempted: result.submission_prize_exempted,
        breakdown_data,
    })
}
