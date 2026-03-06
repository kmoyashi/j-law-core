use std::collections::HashSet;

use pyo3::prelude::*;

use ::j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use ::j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    policy::StandardIncomeTaxPolicy,
};
use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use ::j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use ::j_law_core::LegalDate;
use ::j_law_registry::load_brokerage_fee_params;
use ::j_law_registry::load_consumption_tax_params;
use ::j_law_registry::load_income_tax_params;
use ::j_law_registry::load_stamp_tax_params;

// ═══════════════════════════════════════════════════════════════════════════════
//  消費税
// ═══════════════════════════════════════════════════════════════════════════════

/// 消費税の計算結果。
///
/// Attributes:
///     tax_amount (int): 消費税額（円）
///     amount_with_tax (int): 税込金額（円）
///     amount_without_tax (int): 税抜金額（円）
///     applied_rate_numer (int): 適用税率の分子
///     applied_rate_denom (int): 適用税率の分母
///     is_reduced_rate (bool): 軽減税率が適用されたか
#[pyclass]
struct ConsumptionTaxResult {
    #[pyo3(get)]
    tax_amount: u64,
    #[pyo3(get)]
    amount_with_tax: u64,
    #[pyo3(get)]
    amount_without_tax: u64,
    #[pyo3(get)]
    applied_rate_numer: u64,
    #[pyo3(get)]
    applied_rate_denom: u64,
    #[pyo3(get)]
    is_reduced_rate: bool,
}

#[pymethods]
impl ConsumptionTaxResult {
    fn __repr__(&self) -> String {
        format!(
            "ConsumptionTaxResult(tax_amount={}, amount_with_tax={}, amount_without_tax={}, applied_rate={}/{}, is_reduced_rate={})",
            self.tax_amount,
            self.amount_with_tax,
            self.amount_without_tax,
            self.applied_rate_numer,
            self.applied_rate_denom,
            self.is_reduced_rate,
        )
    }
}

/// 消費税法第29条に基づく消費税額を計算する。
///
/// # 法的根拠
/// 消費税法 第29条（税率）
///
/// Args:
///     amount (int): 課税標準額（税抜き・円）
///     year (int): 基準日（年）
///     month (int): 基準日（月）
///     day (int): 基準日（日）
///     is_reduced_rate (bool): 軽減税率フラグ（デフォルト: False）
///         2019-10-01以降の飲食料品・新聞等に適用される8%軽減税率。
///         WARNING: 対象が軽減税率の適用要件を満たすかの事実認定は呼び出し元の責任。
///
/// Returns:
///     ConsumptionTaxResult
///
/// Raises:
///     ValueError: 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合
#[pyfunction]
#[pyo3(signature = (amount, year, month, day, is_reduced_rate=false))]
fn calc_consumption_tax(
    amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate: bool,
) -> PyResult<ConsumptionTaxResult> {
    let params = load_consumption_tax_params(LegalDate::new(year, month, day))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let mut flags = HashSet::new();
    if is_reduced_rate {
        flags.insert(ConsumptionTaxFlag::ReducedRate);
    }

    let ctx = ConsumptionTaxContext {
        amount,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardConsumptionTaxPolicy),
    };

    let result = calculate_consumption_tax(&ctx, &params)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(ConsumptionTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        amount_with_tax: result.amount_with_tax.as_yen(),
        amount_without_tax: result.amount_without_tax.as_yen(),
        applied_rate_numer: result.applied_rate_numer,
        applied_rate_denom: result.applied_rate_denom,
        is_reduced_rate: result.is_reduced_rate,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  不動産（宅建業法）
// ═══════════════════════════════════════════════════════════════════════════════

/// 1ティアの計算内訳。
#[pyclass(skip_from_py_object)]
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
///     is_seller (bool): 売主側フラグ（デフォルト: False）
///         2018年1月1日〜2024年6月30日の低廉特例は売主のみに適用される。
///         WARNING: 売主・買主の事実認定は呼び出し元の責任。
///
/// Returns:
///     BrokerageFeeResult
///
/// Raises:
///     ValueError: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合
#[pyfunction]
#[pyo3(signature = (price, year, month, day, is_low_cost_vacant_house=false, is_seller=false))]
fn calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
    is_seller: bool,
) -> PyResult<BrokerageFeeResult> {
    let params = load_brokerage_fee_params(LegalDate::new(year, month, day))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

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

// ═══════════════════════════════════════════════════════════════════════════════
//  所得税
// ═══════════════════════════════════════════════════════════════════════════════

/// 所得税の計算内訳（速算表の適用結果）。
#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct IncomeTaxStep {
    #[pyo3(get)]
    label: String,
    /// 課税所得金額（円）
    #[pyo3(get)]
    taxable_income: u64,
    #[pyo3(get)]
    rate_numer: u64,
    #[pyo3(get)]
    rate_denom: u64,
    /// 速算表の控除額（円）
    #[pyo3(get)]
    deduction: u64,
    /// 算出税額（円）
    #[pyo3(get)]
    result: u64,
}

#[pymethods]
impl IncomeTaxStep {
    fn __repr__(&self) -> String {
        format!(
            "IncomeTaxStep(label={:?}, taxable_income={}, rate={}/{}, deduction={}, result={})",
            self.label,
            self.taxable_income,
            self.rate_numer,
            self.rate_denom,
            self.deduction,
            self.result
        )
    }
}

/// 所得税の計算結果。
///
/// Attributes:
///     base_tax (int): 基準所得税額（円）
///     reconstruction_tax (int): 復興特別所得税額（円）
///     total_tax (int): 申告納税額（円・100円未満切り捨て）
///     reconstruction_tax_applied (bool): 復興特別所得税が適用されたか
///     breakdown (list[IncomeTaxStep]): 計算内訳
#[pyclass]
struct IncomeTaxResult {
    #[pyo3(get)]
    base_tax: u64,
    #[pyo3(get)]
    reconstruction_tax: u64,
    #[pyo3(get)]
    total_tax: u64,
    #[pyo3(get)]
    reconstruction_tax_applied: bool,
    #[pyo3(get)]
    breakdown: Vec<IncomeTaxStep>,
}

#[pymethods]
impl IncomeTaxResult {
    fn __repr__(&self) -> String {
        format!(
            "IncomeTaxResult(base_tax={}, reconstruction_tax={}, total_tax={}, reconstruction_tax_applied={})",
            self.base_tax,
            self.reconstruction_tax,
            self.total_tax,
            self.reconstruction_tax_applied,
        )
    }
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項 / 復興財源確保法 第13条
///
/// Args:
///     taxable_income (int): 課税所得金額（円・1,000円未満切り捨て済み）
///     year (int): 対象年度（年）
///     month (int): 基準日（月）
///     day (int): 基準日（日）
///     apply_reconstruction_tax (bool): 復興特別所得税を適用するか（デフォルト: True）
///
/// Returns:
///     IncomeTaxResult
///
/// Raises:
///     ValueError: 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合
#[pyfunction]
#[pyo3(signature = (taxable_income, year, month, day, apply_reconstruction_tax=true))]
fn calc_income_tax(
    taxable_income: u64,
    year: u16,
    month: u8,
    day: u8,
    apply_reconstruction_tax: bool,
) -> PyResult<IncomeTaxResult> {
    let params = load_income_tax_params(LegalDate::new(year, month, day))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

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

    let result = calculate_income_tax(&ctx, &params)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| IncomeTaxStep {
            label: step.label.clone(),
            taxable_income: step.taxable_income,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            deduction: step.deduction,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(IncomeTaxResult {
        base_tax: result.base_tax.as_yen(),
        reconstruction_tax: result.reconstruction_tax.as_yen(),
        total_tax: result.total_tax.as_yen(),
        reconstruction_tax_applied: result.reconstruction_tax_applied,
        breakdown,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  印紙税
// ═══════════════════════════════════════════════════════════════════════════════

/// 印紙税の計算結果。
///
/// Attributes:
///     tax_amount (int): 印紙税額（円）
///     bracket_label (str): 適用されたブラケットの表示名
///     reduced_rate_applied (bool): 軽減税率が適用されたか
#[pyclass]
struct StampTaxResult {
    #[pyo3(get)]
    tax_amount: u64,
    #[pyo3(get)]
    bracket_label: String,
    #[pyo3(get)]
    reduced_rate_applied: bool,
}

#[pymethods]
impl StampTaxResult {
    fn __repr__(&self) -> String {
        format!(
            "StampTaxResult(tax_amount={}, bracket_label={:?}, reduced_rate_applied={})",
            self.tax_amount, self.bracket_label, self.reduced_rate_applied
        )
    }
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書（不動産の譲渡に関する契約書）
/// 租税特別措置法 第91条（軽減措置）
///
/// Args:
///     contract_amount (int): 契約金額（円）
///     year (int): 契約書作成日（年）
///     month (int): 契約書作成日（月）
///     day (int): 契約書作成日（日）
///     is_reduced_rate_applicable (bool): 軽減税率適用フラグ（デフォルト: False）
///         WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任。
///
/// Returns:
///     StampTaxResult
///
/// Raises:
///     ValueError: 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合
#[pyfunction]
#[pyo3(signature = (contract_amount, year, month, day, is_reduced_rate_applicable=false))]
fn calc_stamp_tax(
    contract_amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate_applicable: bool,
) -> PyResult<StampTaxResult> {
    let params = load_stamp_tax_params(LegalDate::new(year, month, day))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let mut flags = HashSet::new();
    if is_reduced_rate_applicable {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }

    let ctx = StampTaxContext {
        contract_amount,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    let result = calculate_stamp_tax(&ctx, &params)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(StampTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        bracket_label: result.bracket_label,
        reduced_rate_applied: result.reduced_rate_applied,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
//  モジュール定義
// ═══════════════════════════════════════════════════════════════════════════════

/// 消費税ドメインサブモジュール。
fn register_consumption_tax(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "consumption_tax")?;
    m.add_class::<ConsumptionTaxResult>()?;
    m.add_function(wrap_pyfunction!(calc_consumption_tax, &m)?)?;
    parent.add_submodule(&m)?;
    // sys.modules に登録して `from j_law_python.consumption_tax import ...` を有効にする
    py.import("sys")?
        .getattr("modules")?
        .set_item("j_law_python.consumption_tax", &m)?;
    Ok(())
}

/// 不動産ドメイン（宅建業法）サブモジュール。
fn register_real_estate(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "real_estate")?;
    m.add_class::<BrokerageFeeResult>()?;
    m.add_class::<BreakdownStep>()?;
    m.add_function(wrap_pyfunction!(calc_brokerage_fee, &m)?)?;
    parent.add_submodule(&m)?;
    // sys.modules に登録して `from j_law_python.real_estate import ...` を有効にする
    py.import("sys")?
        .getattr("modules")?
        .set_item("j_law_python.real_estate", &m)?;
    Ok(())
}

/// 所得税ドメインサブモジュール。
fn register_income_tax(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "income_tax")?;
    m.add_class::<IncomeTaxResult>()?;
    m.add_class::<IncomeTaxStep>()?;
    m.add_function(wrap_pyfunction!(calc_income_tax, &m)?)?;
    parent.add_submodule(&m)?;
    // sys.modules に登録して `from j_law_python.income_tax import ...` を有効にする
    py.import("sys")?
        .getattr("modules")?
        .set_item("j_law_python.income_tax", &m)?;
    Ok(())
}

/// 印紙税ドメインサブモジュール。
fn register_stamp_tax(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "stamp_tax")?;
    m.add_class::<StampTaxResult>()?;
    m.add_function(wrap_pyfunction!(calc_stamp_tax, &m)?)?;
    parent.add_submodule(&m)?;
    // sys.modules に登録して `from j_law_python.stamp_tax import ...` を有効にする
    py.import("sys")?
        .getattr("modules")?
        .set_item("j_law_python.stamp_tax", &m)?;
    Ok(())
}

/// j_law_python Python モジュール。
///
/// 使用例::
///
///     import j_law_python
///     result = j_law_python.real_estate.calc_brokerage_fee(
///         price=5_000_000, year=2024, month=8, day=1
///     )
///     print(result.total_with_tax)     # 231000
///
///     result = j_law_python.income_tax.calc_income_tax(
///         taxable_income=5_000_000, year=2024, month=1, day=1
///     )
///     print(result.total_tax)          # 584500
///
///     result = j_law_python.stamp_tax.calc_stamp_tax(
///         contract_amount=5_000_000, year=2024, month=8, day=1
///     )
///     print(result.tax_amount)         # 2000
#[pymodule]
fn j_law_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_consumption_tax(m)?;
    register_real_estate(m)?;
    register_income_tax(m)?;
    register_stamp_tax(m)?;
    Ok(())
}
