use j_law_uniffi::{
    calc_brokerage_fee as uniffi_calc_brokerage_fee,
    calc_consumption_tax as uniffi_calc_consumption_tax, calc_income_tax as uniffi_calc_income_tax,
    calc_stamp_tax as uniffi_calc_stamp_tax, UniBrokerageFeeResult, UniConsumptionTaxResult,
    UniError, UniIncomeTaxResult, UniStampTaxResult,
};

use magnus::value::ReprValue;
use magnus::{function, method, Error, Module, RArray, Ruby, Symbol, Value};

fn into_runtime_error(e: UniError) -> Error {
    Error::new(magnus::exception::runtime_error(), e.to_string())
}

/// Ruby の Date / DateTime オブジェクトから (year, month, day) を取得する。
///
/// Date / DateTime 以外のオブジェクトを渡した場合は TypeError を送出する。
fn extract_date(date: Value) -> Result<(u16, u8, u8), Error> {
    // SAFETY: classname() は Magnus の ReprValue トレイトが提供する。
    // Ruby の GIL 保持下でのみ呼ばれるため安全。
    let class_name = unsafe { date.classname() };
    if class_name != "Date" && class_name != "DateTime" {
        return Err(Error::new(
            magnus::exception::type_error(),
            format!(
                "date には Date または DateTime を指定してください (got {})",
                class_name
            ),
        ));
    }
    let year: i32 = date.funcall("year", ())?;
    let month: i32 = date.funcall("month", ())?;
    let day: i32 = date.funcall("day", ())?;
    Ok((year as u16, month as u8, day as u8))
}

// ─── 消費税 Ruby公開型 ──────────────────────────────────────────────────────────

/// 消費税の計算結果。
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::ConsumptionTax::ConsumptionTaxResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbConsumptionTaxResult(UniConsumptionTaxResult);

impl RbConsumptionTaxResult {
    fn tax_amount(&self) -> u64 {
        self.0.tax_amount
    }

    fn amount_with_tax(&self) -> u64 {
        self.0.amount_with_tax
    }

    fn amount_without_tax(&self) -> u64 {
        self.0.amount_without_tax
    }

    fn applied_rate_numer(&self) -> u64 {
        self.0.applied_rate_numer
    }

    fn applied_rate_denom(&self) -> u64 {
        self.0.applied_rate_denom
    }

    fn is_reduced_rate(&self) -> bool {
        self.0.is_reduced_rate
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawRuby::ConsumptionTax::ConsumptionTaxResult tax_amount={} amount_with_tax={} amount_without_tax={} applied_rate={}/{} is_reduced_rate={}>",
            self.0.tax_amount,
            self.0.amount_with_tax,
            self.0.amount_without_tax,
            self.0.applied_rate_numer,
            self.0.applied_rate_denom,
            self.0.is_reduced_rate,
        )
    }
}

// ─── 消費税 Ruby公開関数 ────────────────────────────────────────────────────────

/// 消費税法第29条に基づく消費税額を計算する。
///
/// @param amount [Integer] 課税標準額（税抜き・円）
/// @param date [Date] 基準日
/// @param is_reduced_rate [true, false] 軽減税率フラグ
/// @return [JLawRuby::ConsumptionTax::ConsumptionTaxResult]
/// @raise [TypeError] date が Date / DateTime 以外の場合
/// @raise [RuntimeError] 計算エラーが発生した場合
fn calc_consumption_tax(
    amount: u64,
    date: Value,
    is_reduced_rate: bool,
) -> Result<RbConsumptionTaxResult, Error> {
    let (year, month, day) = extract_date(date)?;
    let result = uniffi_calc_consumption_tax(amount, year, month, day, is_reduced_rate)
        .map_err(into_runtime_error)?;
    Ok(RbConsumptionTaxResult(result))
}

// ─── 媒介報酬 Ruby公開型 ────────────────────────────────────────────────────────

/// 媒介報酬の計算結果。
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::RealEstate::BrokerageFeeResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbBrokerageFeeResult(UniBrokerageFeeResult);

impl RbBrokerageFeeResult {
    fn total_without_tax(&self) -> u64 {
        self.0.total_without_tax
    }

    fn total_with_tax(&self) -> u64 {
        self.0.total_with_tax
    }

    fn tax_amount(&self) -> u64 {
        self.0.tax_amount
    }

    fn low_cost_special_applied(&self) -> bool {
        self.0.low_cost_special_applied
    }

    /// 各ティアの内訳を Hash の Array で返す。
    fn breakdown(&self) -> Result<RArray, Error> {
        // SAFETY: Magnus が #[magnus::wrap] で wrap したオブジェクトのメソッドは
        // Ruby の GIL 保持下で呼ばれるため、Ruby ランタイムは必ず初期化済みである。
        let ruby = unsafe { Ruby::get_unchecked() };
        let arr = ruby.ary_new();
        for step in &self.0.breakdown {
            let hash = ruby.hash_new();
            hash.aset(Symbol::new("label"), step.label.as_str())?;
            hash.aset(Symbol::new("base_amount"), step.base_amount)?;
            hash.aset(Symbol::new("rate_numer"), step.rate_numer)?;
            hash.aset(Symbol::new("rate_denom"), step.rate_denom)?;
            hash.aset(Symbol::new("result"), step.result)?;
            arr.push(hash)?;
        }
        Ok(arr)
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawRuby::RealEstate::BrokerageFeeResult total_without_tax={} total_with_tax={} tax_amount={} low_cost_special_applied={}>",
            self.0.total_without_tax,
            self.0.total_with_tax,
            self.0.tax_amount,
            self.0.low_cost_special_applied,
        )
    }
}

// ─── 媒介報酬 Ruby公開関数 ──────────────────────────────────────────────────────

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// @param price [Integer] 売買価格（円）
/// @param date [Date] 基準日
/// @param is_low_cost_vacant_house [true, false] 低廉な空き家特例フラグ
/// @param is_seller [true, false] 売主側フラグ
/// @return [JLawRuby::RealEstate::BrokerageFeeResult]
/// @raise [TypeError] date が Date / DateTime 以外の場合
/// @raise [RuntimeError] 計算エラーが発生した場合
fn calc_brokerage_fee(
    price: u64,
    date: Value,
    is_low_cost_vacant_house: bool,
    is_seller: bool,
) -> Result<RbBrokerageFeeResult, Error> {
    let (year, month, day) = extract_date(date)?;
    let result =
        uniffi_calc_brokerage_fee(price, year, month, day, is_low_cost_vacant_house, is_seller)
            .map_err(into_runtime_error)?;
    Ok(RbBrokerageFeeResult(result))
}

// ─── 所得税 Ruby公開型 ──────────────────────────────────────────────────────────

/// 所得税の計算結果。
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::IncomeTax::IncomeTaxResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbIncomeTaxResult(UniIncomeTaxResult);

impl RbIncomeTaxResult {
    fn base_tax(&self) -> u64 {
        self.0.base_tax
    }

    fn reconstruction_tax(&self) -> u64 {
        self.0.reconstruction_tax
    }

    fn total_tax(&self) -> u64 {
        self.0.total_tax
    }

    fn reconstruction_tax_applied(&self) -> bool {
        self.0.reconstruction_tax_applied
    }

    /// 計算内訳を Hash の Array で返す。
    fn breakdown(&self) -> Result<RArray, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let arr = ruby.ary_new();
        for step in &self.0.breakdown {
            let hash = ruby.hash_new();
            hash.aset(Symbol::new("label"), step.label.as_str())?;
            hash.aset(Symbol::new("taxable_income"), step.taxable_income)?;
            hash.aset(Symbol::new("rate_numer"), step.rate_numer)?;
            hash.aset(Symbol::new("rate_denom"), step.rate_denom)?;
            hash.aset(Symbol::new("deduction"), step.deduction)?;
            hash.aset(Symbol::new("result"), step.result)?;
            arr.push(hash)?;
        }
        Ok(arr)
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawRuby::IncomeTax::IncomeTaxResult base_tax={} reconstruction_tax={} total_tax={} reconstruction_tax_applied={}>",
            self.0.base_tax,
            self.0.reconstruction_tax,
            self.0.total_tax,
            self.0.reconstruction_tax_applied,
        )
    }
}

// ─── 所得税 Ruby公開関数 ────────────────────────────────────────────────────────

/// 所得税法第89条に基づく所得税額を計算する。
///
/// @param taxable_income [Integer] 課税所得金額（円）
/// @param date [Date] 基準日
/// @param apply_reconstruction_tax [true, false] 復興特別所得税を適用するか
/// @return [JLawRuby::IncomeTax::IncomeTaxResult]
/// @raise [TypeError] date が Date / DateTime 以外の場合
/// @raise [RuntimeError] 計算エラーが発生した場合
fn calc_income_tax(
    taxable_income: u64,
    date: Value,
    apply_reconstruction_tax: bool,
) -> Result<RbIncomeTaxResult, Error> {
    let (year, month, day) = extract_date(date)?;
    let result = uniffi_calc_income_tax(taxable_income, year, month, day, apply_reconstruction_tax)
        .map_err(into_runtime_error)?;
    Ok(RbIncomeTaxResult(result))
}

// ─── 印紙税 Ruby公開型 ──────────────────────────────────────────────────────────

/// 印紙税の計算結果。
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::StampTax::StampTaxResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbStampTaxResult(UniStampTaxResult);

impl RbStampTaxResult {
    fn tax_amount(&self) -> u64 {
        self.0.tax_amount
    }

    fn bracket_label(&self) -> String {
        self.0.bracket_label.clone()
    }

    fn reduced_rate_applied(&self) -> bool {
        self.0.reduced_rate_applied
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawRuby::StampTax::StampTaxResult tax_amount={} bracket_label={:?} reduced_rate_applied={}>",
            self.0.tax_amount,
            self.0.bracket_label,
            self.0.reduced_rate_applied,
        )
    }
}

// ─── 印紙税 Ruby公開関数 ────────────────────────────────────────────────────────

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// @param contract_amount [Integer] 契約金額（円）
/// @param date [Date] 契約書作成日
/// @param is_reduced_rate_applicable [true, false] 軽減税率適用フラグ
/// @return [JLawRuby::StampTax::StampTaxResult]
/// @raise [TypeError] date が Date / DateTime 以外の場合
/// @raise [RuntimeError] 計算エラーが発生した場合
fn calc_stamp_tax(
    contract_amount: u64,
    date: Value,
    is_reduced_rate_applicable: bool,
) -> Result<RbStampTaxResult, Error> {
    let (year, month, day) = extract_date(date)?;
    let result = uniffi_calc_stamp_tax(
        contract_amount,
        year,
        month,
        day,
        is_reduced_rate_applicable,
    )
    .map_err(into_runtime_error)?;
    Ok(RbStampTaxResult(result))
}

// ─── モジュール定義 ────────────────────────────────────────────────────────────

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let j_law_core = ruby.define_module("JLawRuby")?;

    // ─── 消費税 ───────────────────────────────────────────────────────────────
    let consumption_tax = j_law_core.define_module("ConsumptionTax")?;

    let consumption_tax_result_class =
        consumption_tax.define_class("ConsumptionTaxResult", ruby.class_object())?;
    consumption_tax_result_class
        .define_method("tax_amount", method!(RbConsumptionTaxResult::tax_amount, 0))?;
    consumption_tax_result_class.define_method(
        "amount_with_tax",
        method!(RbConsumptionTaxResult::amount_with_tax, 0),
    )?;
    consumption_tax_result_class.define_method(
        "amount_without_tax",
        method!(RbConsumptionTaxResult::amount_without_tax, 0),
    )?;
    consumption_tax_result_class.define_method(
        "applied_rate_numer",
        method!(RbConsumptionTaxResult::applied_rate_numer, 0),
    )?;
    consumption_tax_result_class.define_method(
        "applied_rate_denom",
        method!(RbConsumptionTaxResult::applied_rate_denom, 0),
    )?;
    consumption_tax_result_class.define_method(
        "is_reduced_rate?",
        method!(RbConsumptionTaxResult::is_reduced_rate, 0),
    )?;
    consumption_tax_result_class
        .define_method("inspect", method!(RbConsumptionTaxResult::inspect, 0))?;
    consumption_tax_result_class
        .define_method("to_s", method!(RbConsumptionTaxResult::inspect, 0))?;

    consumption_tax
        .define_module_function("calc_consumption_tax", function!(calc_consumption_tax, 3))?;

    // ─── 不動産 ───────────────────────────────────────────────────────────────
    let real_estate = j_law_core.define_module("RealEstate")?;

    let result_class = real_estate.define_class("BrokerageFeeResult", ruby.class_object())?;
    result_class.define_method(
        "total_without_tax",
        method!(RbBrokerageFeeResult::total_without_tax, 0),
    )?;
    result_class.define_method(
        "total_with_tax",
        method!(RbBrokerageFeeResult::total_with_tax, 0),
    )?;
    result_class.define_method("tax_amount", method!(RbBrokerageFeeResult::tax_amount, 0))?;
    result_class.define_method(
        "low_cost_special_applied?",
        method!(RbBrokerageFeeResult::low_cost_special_applied, 0),
    )?;
    result_class.define_method("breakdown", method!(RbBrokerageFeeResult::breakdown, 0))?;
    result_class.define_method("inspect", method!(RbBrokerageFeeResult::inspect, 0))?;
    result_class.define_method("to_s", method!(RbBrokerageFeeResult::inspect, 0))?;

    real_estate.define_module_function("calc_brokerage_fee", function!(calc_brokerage_fee, 4))?;

    // ─── 所得税 ───────────────────────────────────────────────────────────────
    let income_tax = j_law_core.define_module("IncomeTax")?;

    let income_tax_result_class =
        income_tax.define_class("IncomeTaxResult", ruby.class_object())?;
    income_tax_result_class.define_method("base_tax", method!(RbIncomeTaxResult::base_tax, 0))?;
    income_tax_result_class.define_method(
        "reconstruction_tax",
        method!(RbIncomeTaxResult::reconstruction_tax, 0),
    )?;
    income_tax_result_class.define_method("total_tax", method!(RbIncomeTaxResult::total_tax, 0))?;
    income_tax_result_class.define_method(
        "reconstruction_tax_applied?",
        method!(RbIncomeTaxResult::reconstruction_tax_applied, 0),
    )?;
    income_tax_result_class.define_method("breakdown", method!(RbIncomeTaxResult::breakdown, 0))?;
    income_tax_result_class.define_method("inspect", method!(RbIncomeTaxResult::inspect, 0))?;
    income_tax_result_class.define_method("to_s", method!(RbIncomeTaxResult::inspect, 0))?;

    income_tax.define_module_function("calc_income_tax", function!(calc_income_tax, 3))?;

    // ─── 印紙税 ───────────────────────────────────────────────────────────────
    let stamp_tax = j_law_core.define_module("StampTax")?;

    let stamp_tax_result_class = stamp_tax.define_class("StampTaxResult", ruby.class_object())?;
    stamp_tax_result_class.define_method("tax_amount", method!(RbStampTaxResult::tax_amount, 0))?;
    stamp_tax_result_class
        .define_method("bracket_label", method!(RbStampTaxResult::bracket_label, 0))?;
    stamp_tax_result_class.define_method(
        "reduced_rate_applied?",
        method!(RbStampTaxResult::reduced_rate_applied, 0),
    )?;
    stamp_tax_result_class.define_method("inspect", method!(RbStampTaxResult::inspect, 0))?;
    stamp_tax_result_class.define_method("to_s", method!(RbStampTaxResult::inspect, 0))?;

    stamp_tax.define_module_function("calc_stamp_tax", function!(calc_stamp_tax, 3))?;

    Ok(())
}
