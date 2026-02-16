use std::collections::HashSet;

use js_sys::Array;
use wasm_bindgen::prelude::*;

use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
    RealEstateFlag,
};
use ::j_law_registry::load_brokerage_fee_params;

// ─── JavaScript公開型 ──────────────────────────────────────────────────────────

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
    // breakdown は Vec<T> をそのまま JS に返せないため、内部データとして保持し
    // breakdown() メソッドで Plain JS オブジェクトの配列に変換して返す。
    breakdown_data: Vec<BreakdownStepData>,
}

/// breakdown() で使う内部データ型（wasm_bindgen 不要）。
struct BreakdownStepData {
    label: String,
    base_amount: u32,
    rate_numer: u32,
    rate_denom: u32,
    result: u32,
}

#[wasm_bindgen]
impl BrokerageFeeResult {
    /// 税抜合計額（円）
    #[wasm_bindgen(getter, js_name = "totalWithoutTax")]
    pub fn total_without_tax(&self) -> u32 {
        self.total_without_tax
    }

    /// 税込合計額（円）
    #[wasm_bindgen(getter, js_name = "totalWithTax")]
    pub fn total_with_tax(&self) -> u32 {
        self.total_with_tax
    }

    /// 消費税額（円）
    #[wasm_bindgen(getter, js_name = "taxAmount")]
    pub fn tax_amount(&self) -> u32 {
        self.tax_amount
    }

    /// 低廉な空き家特例が適用されたか
    #[wasm_bindgen(getter, js_name = "lowCostSpecialApplied")]
    pub fn low_cost_special_applied(&self) -> bool {
        self.low_cost_special_applied
    }

    /// 各ティアの計算内訳を Plain JS オブジェクトの配列で返す。
    ///
    /// 各オブジェクトのプロパティ:
    /// - `label`: string
    /// - `baseAmount`: number（ティア対象金額・円）
    /// - `rateNumer`: number
    /// - `rateDenom`: number
    /// - `result`: number（ティア計算結果・円）
    pub fn breakdown(&self) -> Array {
        self.breakdown_data
            .iter()
            .map(|step| {
                let obj = js_sys::Object::new();
                // plain object への文字列キー設定は失敗しないため _ で無視する
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

// ─── JavaScript公開関数 ────────────────────────────────────────────────────────

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項 / 国土交通省告示
///
/// @param price - 売買価格（円）※ u32 上限: 約42.9億円
/// @param year - 基準日（年）
/// @param month - 基準日（月）
/// @param day - 基準日（日）
/// @param isLowCostVacantHouse - 低廉な空き家特例フラグ（デフォルト: false）
///   WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
/// @returns BrokerageFeeResult
/// @throws 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合
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
