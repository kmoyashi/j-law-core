use std::collections::HashSet;

use pyo3::prelude::*;

use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
    RealEstateFlag,
};
use ::j_law_registry::load_brokerage_fee_params;

// ─── Python公開型 ──────────────────────────────────────────────────────────────

/// 1ティアの計算内訳。
#[pyclass]
#[derive(Clone)]
struct BreakdownStep {
    #[pyo3(get)]
    label: String,
    /// ティア対象金額（円）
    #[pyo3(get)]
    base_amount: u64,
    #[pyo3(get)]
    rate_numer: u64,
    #[pyo3(get)]
    rate_denom: u64,
    /// ティア計算結果（円・端数切捨て済み）
    #[pyo3(get)]
    result: u64,
}

#[pymethods]
impl BreakdownStep {
    fn __repr__(&self) -> String {
        format!(
            "BreakdownStep(label={:?}, base_amount={}, rate={}/{}, result={})",
            self.label, self.base_amount, self.rate_numer, self.rate_denom, self.result
        )
    }
}

/// 媒介報酬の計算結果。
///
/// Attributes:
///     total_without_tax (int): 税抜合計額（円）
///     total_with_tax (int): 税込合計額（円）
///     tax_amount (int): 消費税額（円）
///     low_cost_special_applied (bool): 低廉な空き家特例が適用されたか
///     breakdown (list[BreakdownStep]): 各ティアの計算内訳
#[pyclass]
struct BrokerageFeeResult {
    #[pyo3(get)]
    total_without_tax: u64,
    #[pyo3(get)]
    total_with_tax: u64,
    #[pyo3(get)]
    tax_amount: u64,
    #[pyo3(get)]
    low_cost_special_applied: bool,
    #[pyo3(get)]
    breakdown: Vec<BreakdownStep>,
}

#[pymethods]
impl BrokerageFeeResult {
    fn __repr__(&self) -> String {
        format!(
            "BrokerageFeeResult(total_without_tax={}, total_with_tax={}, tax_amount={}, low_cost_special_applied={})",
            self.total_without_tax,
            self.total_with_tax,
            self.tax_amount,
            self.low_cost_special_applied,
        )
    }
}

// ─── Python公開関数 ────────────────────────────────────────────────────────────

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項 / 国土交通省告示
///
/// Args:
///     price (int): 売買価格（円）
///     year (int): 基準日（年）
///     month (int): 基準日（月）
///     day (int): 基準日（日）
///     is_low_cost_vacant_house (bool): 低廉な空き家特例フラグ（デフォルト: False）
///         WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
///
/// Returns:
///     BrokerageFeeResult
///
/// Raises:
///     ValueError: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合
#[pyfunction]
#[pyo3(signature = (price, year, month, day, is_low_cost_vacant_house=false))]
fn calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
) -> PyResult<BrokerageFeeResult> {
    let params = load_brokerage_fee_params((year, month, day))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let mut flags = HashSet::new();
    if is_low_cost_vacant_house {
        flags.insert(RealEstateFlag::IsLowCostVacantHouse);
    }

    let ctx = RealEstateContext {
        price,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardMliitPolicy),
    };

    let result = calculate_brokerage_fee(&ctx, &params)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| BreakdownStep {
            label: step.label.clone(),
            base_amount: step.base_amount,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(BrokerageFeeResult {
        total_without_tax: result.total_without_tax.as_yen(),
        total_with_tax: result.total_with_tax.as_yen(),
        tax_amount: result.tax_amount.as_yen(),
        low_cost_special_applied: result.low_cost_special_applied,
        breakdown,
    })
}

// ─── モジュール定義 ────────────────────────────────────────────────────────────

/// 不動産ドメイン（宅建業法）サブモジュール。
fn register_real_estate(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new_bound(parent.py(), "real_estate")?;
    m.add_class::<BrokerageFeeResult>()?;
    m.add_class::<BreakdownStep>()?;
    m.add_function(wrap_pyfunction!(calc_brokerage_fee, &m)?)?;
    parent.add_submodule(&m)?;
    Ok(())
}

/// j_law_core Python モジュール。
///
/// 使用例::
///
///     import j_law_core
///     result = j_law_core.real_estate.calc_brokerage_fee(
///         price=5_000_000, year=2024, month=8, day=1
///     )
///     print(result.total_without_tax)  # 210000
///     print(result.total_with_tax)     # 231000
#[pymodule]
fn j_law_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_real_estate(m)?;
    Ok(())
}
