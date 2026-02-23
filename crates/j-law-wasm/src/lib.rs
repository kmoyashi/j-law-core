use std::collections::HashSet;

use js_sys::Array;
use wasm_bindgen::prelude::*;

use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
    RealEstateFlag,
};
use ::j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    policy::StandardIncomeTaxPolicy,
};
use ::j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use ::j_law_registry::load_brokerage_fee_params;
use ::j_law_registry::load_income_tax_params;
use ::j_law_registry::load_stamp_tax_params;

// ═══════════════════════════════════════════════════════════════════════════════
//  不動産（宅建業法）
// ═══════════════════════════════════════════════════════════════════════════════

/// 媒介報酬の計算結果。
///
/// Properties:
/// - `totalWithoutTax`: 税抜合計額（円）
/// - `totalWithTax`: 税込合計額（円）
/// - `taxAmount`: 消費税額（円）
/// - `lowCostSpecialApplied`: 低廉な空き家特例が適用されたか
/// - `breakdown()`: 各ティアの計算内訳（`{ label, baseAmount, rateNumer, rateDenom, result }[]`）
///
/// NOTE: 金額フィールドは u32（最大約42.9億円）。
/// JavaScript の Number 精度制約（53bit整数）との互換性を保つため意図的に u32 を使用している。
/// u64 が必要な取引には wasm-bindgen の BigInt 対応バインディングを別途検討すること。
#[wasm_bindgen]
pub struct BrokerageFeeResult {
    total_without_tax: u32,
    total_with_tax: u32,
    tax_amount: u32,
    low_cost_special_applied: bool,
    breakdown_data: Vec<BreakdownStepData>,
}

struct BreakdownStepData {
    label: String,
    base_amount: u32,
    rate_numer: u32,
    rate_denom: u32,
    result: u32,
}

#[wasm_bindgen]
impl BrokerageFeeResult {
    #[wasm_bindgen(getter, js_name = "totalWithoutTax")]
    pub fn total_without_tax(&self) -> u32 {
        self.total_without_tax
    }

    #[wasm_bindgen(getter, js_name = "totalWithTax")]
    pub fn total_with_tax(&self) -> u32 {
        self.total_with_tax
    }

    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u32 {
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
                    &JsValue::from_f64(step.base_amount as f64),
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
                    &JsValue::from_f64(step.result as f64),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

#[wasm_bindgen(js_name = "calcBrokerageFee")]
pub fn calc_brokerage_fee(
    price: u32,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
) -> Result<BrokerageFeeResult, JsValue> {
    let params = load_brokerage_fee_params((year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if is_low_cost_vacant_house {
        flags.insert(RealEstateFlag::IsLowCostVacantHouse);
    }

    let ctx = RealEstateContext {
        price: price as u64,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardMliitPolicy),
    };

    let result = calculate_brokerage_fee(&ctx, &params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|step| BreakdownStepData {
            label: step.label.clone(),
            base_amount: step.base_amount as u32,
            rate_numer: step.rate_numer as u32,
            rate_denom: step.rate_denom as u32,
            result: step.result.as_yen() as u32,
        })
        .collect();

    Ok(BrokerageFeeResult {
        total_without_tax: result.total_without_tax.as_yen() as u32,
        total_with_tax: result.total_with_tax.as_yen() as u32,
        tax_amount: result.tax_amount.as_yen() as u32,
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
/// - `baseTax`: 基準所得税額（円）
/// - `reconstructionTax`: 復興特別所得税額（円）
/// - `totalTax`: 申告納税額（円・100円未満切り捨て）
/// - `reconstructionTaxApplied`: 復興特別所得税が適用されたか
/// - `breakdown()`: 計算内訳（`{ label, taxableIncome, rateNumer, rateDenom, deduction, result }[]`）
#[wasm_bindgen]
pub struct IncomeTaxResult {
    base_tax: u32,
    reconstruction_tax: u32,
    total_tax: u32,
    reconstruction_tax_applied: bool,
    breakdown_data: Vec<IncomeTaxStepData>,
}

struct IncomeTaxStepData {
    label: String,
    taxable_income: u32,
    rate_numer: u32,
    rate_denom: u32,
    deduction: u32,
    result: u32,
}

#[wasm_bindgen]
impl IncomeTaxResult {
    #[wasm_bindgen(getter, js_name = "baseTax")]
    pub fn base_tax(&self) -> u32 {
        self.base_tax
    }

    #[wasm_bindgen(getter, js_name = "reconstructionTax")]
    pub fn reconstruction_tax(&self) -> u32 {
        self.reconstruction_tax
    }

    #[wasm_bindgen(getter, js_name = "totalTax")]
    pub fn total_tax(&self) -> u32 {
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
                    &JsValue::from_f64(step.taxable_income as f64),
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
                    &JsValue::from_f64(step.deduction as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("result"),
                    &JsValue::from_f64(step.result as f64),
                );
                JsValue::from(obj)
            })
            .collect()
    }
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// @param taxableIncome - 課税所得金額（円）
/// @param year - 対象年度（年）
/// @param month - 基準日（月）
/// @param day - 基準日（日）
/// @param applyReconstructionTax - 復興特別所得税を適用するか
/// @returns IncomeTaxResult
/// @throws 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合
#[wasm_bindgen(js_name = "calcIncomeTax")]
pub fn calc_income_tax(
    taxable_income: u32,
    year: u16,
    month: u8,
    day: u8,
    apply_reconstruction_tax: bool,
) -> Result<IncomeTaxResult, JsValue> {
    let params = load_income_tax_params((year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if apply_reconstruction_tax {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }

    let ctx = IncomeTaxContext {
        taxable_income: taxable_income as u64,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let result = calculate_income_tax(&ctx, &params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let breakdown_data = result
        .breakdown
        .iter()
        .map(|step| IncomeTaxStepData {
            label: step.label.clone(),
            taxable_income: step.taxable_income as u32,
            rate_numer: step.rate_numer as u32,
            rate_denom: step.rate_denom as u32,
            deduction: step.deduction as u32,
            result: step.result.as_yen() as u32,
        })
        .collect();

    Ok(IncomeTaxResult {
        base_tax: result.base_tax.as_yen() as u32,
        reconstruction_tax: result.reconstruction_tax.as_yen() as u32,
        total_tax: result.total_tax.as_yen() as u32,
        reconstruction_tax_applied: result.reconstruction_tax_applied,
        breakdown_data,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  印紙税
// ═══════════════════════════════════════════════════════════════════════════════

/// 印紙税の計算結果。
///
/// Properties:
/// - `taxAmount`: 印紙税額（円）
/// - `bracketLabel`: 適用されたブラケットの表示名
/// - `reducedRateApplied`: 軽減税率が適用されたか
#[wasm_bindgen]
pub struct StampTaxResult {
    tax_amount: u32,
    bracket_label: String,
    reduced_rate_applied: bool,
}

#[wasm_bindgen]
impl StampTaxResult {
    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u32 {
        self.tax_amount
    }

    #[wasm_bindgen(getter, js_name = "bracketLabel")]
    pub fn bracket_label(&self) -> String {
        self.bracket_label.clone()
    }

    #[wasm_bindgen(getter, js_name = "reducedRateApplied")]
    pub fn reduced_rate_applied(&self) -> bool {
        self.reduced_rate_applied
    }
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// @param contractAmount - 契約金額（円）
/// @param year - 契約書作成日（年）
/// @param month - 契約書作成日（月）
/// @param day - 契約書作成日（日）
/// @param isReducedRateApplicable - 軽減税率適用フラグ
/// @returns StampTaxResult
/// @throws 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合
#[wasm_bindgen(js_name = "calcStampTax")]
pub fn calc_stamp_tax(
    contract_amount: u32,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate_applicable: bool,
) -> Result<StampTaxResult, JsValue> {
    let params = load_stamp_tax_params((year, month, day))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut flags = HashSet::new();
    if is_reduced_rate_applicable {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }

    let ctx = StampTaxContext {
        contract_amount: contract_amount as u64,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    let result = calculate_stamp_tax(&ctx, &params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(StampTaxResult {
        tax_amount: result.tax_amount.as_yen() as u32,
        bracket_label: result.bracket_label,
        reduced_rate_applied: result.reduced_rate_applied,
    })
}
