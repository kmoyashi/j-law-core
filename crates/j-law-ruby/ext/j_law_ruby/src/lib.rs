use std::collections::HashSet;

use magnus::{function, method, Error, Module, RArray, Ruby, Symbol};

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
use ::j_law_registry::load_brokerage_fee_params;
use ::j_law_registry::load_income_tax_params;
use ::j_law_registry::load_stamp_tax_params;

fn into_runtime_error<E: std::fmt::Debug>(e: E) -> Error {
    Error::new(magnus::exception::runtime_error(), format!("{e:?}"))
}

// ─── BreakdownStep（内部データ型） ──────────────────────────────────────────────

struct BreakdownStepData {
    label: String,
    base_amount: u64,
    rate_numer: u64,
    rate_denom: u64,
    result: u64,
}

// ─── Ruby公開型 ────────────────────────────────────────────────────────────────

/// 媒介報酬の計算結果。
///
/// メソッド:
/// - `total_without_tax` → Integer（税抜合計額・円）
/// - `total_with_tax`    → Integer（税込合計額・円）
/// - `tax_amount`        → Integer（消費税額・円）
/// - `low_cost_special_applied?` → true/false
/// - `breakdown` → Array<Hash>（各ティアの内訳）
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::RealEstate::BrokerageFeeResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbBrokerageFeeResult {
    total_without_tax: u64,
    total_with_tax: u64,
    tax_amount: u64,
    low_cost_special_applied: bool,
    breakdown: Vec<BreakdownStepData>,
}

impl RbBrokerageFeeResult {
    fn total_without_tax(&self) -> u64 {
        self.total_without_tax
    }

    fn total_with_tax(&self) -> u64 {
        self.total_with_tax
    }

    fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    fn low_cost_special_applied(&self) -> bool {
        self.low_cost_special_applied
    }

    /// 各ティアの内訳を Hash の Array で返す。
    ///
    /// 各 Hash のキー:
    /// - `:label`       String
    /// - `:base_amount` Integer（ティア対象金額・円）
    /// - `:rate_numer`  Integer
    /// - `:rate_denom`  Integer
    /// - `:result`      Integer（ティア計算結果・円）
    fn breakdown(&self) -> Result<RArray, Error> {
        // SAFETY: Magnus が #[magnus::wrap] で wrap したオブジェクトのメソッドは
        // Ruby の GIL 保持下で呼ばれるため、Ruby ランタイムは必ず初期化済みである。
        let ruby = unsafe { Ruby::get_unchecked() };
        let arr = ruby.ary_new();
        for step in &self.breakdown {
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
            self.total_without_tax,
            self.total_with_tax,
            self.tax_amount,
            self.low_cost_special_applied,
        )
    }
}

// ─── Ruby公開関数 ──────────────────────────────────────────────────────────────

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項 / 国土交通省告示
///
/// @param price [Integer] 売買価格（円）
/// @param year  [Integer] 基準日（年）
/// @param month [Integer] 基準日（月）
/// @param day   [Integer] 基準日（日）
/// @param is_low_cost_vacant_house [true, false] 低廉な空き家特例フラグ
///   WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
/// @return [JLawRuby::RealEstate::BrokerageFeeResult]
/// @raise [RuntimeError] 対象日に有効な法令パラメータが存在しない場合
fn calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
) -> Result<RbBrokerageFeeResult, Error> {
    let params = load_brokerage_fee_params((year, month, day)).map_err(into_runtime_error)?;

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

    let result = calculate_brokerage_fee(&ctx, &params).map_err(into_runtime_error)?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| BreakdownStepData {
            label: step.label.clone(),
            base_amount: step.base_amount,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(RbBrokerageFeeResult {
        total_without_tax: result.total_without_tax.as_yen(),
        total_with_tax: result.total_with_tax.as_yen(),
        tax_amount: result.tax_amount.as_yen(),
        low_cost_special_applied: result.low_cost_special_applied,
        breakdown,
    })
}

// ─── 所得税 内部データ型 ──────────────────────────────────────────────────────

struct IncomeTaxStepData {
    label: String,
    taxable_income: u64,
    rate_numer: u64,
    rate_denom: u64,
    deduction: u64,
    result: u64,
}

// ─── 所得税 Ruby公開型 ──────────────────────────────────────────────────────────

/// 所得税の計算結果。
///
/// メソッド:
/// - `base_tax` → Integer（基準所得税額・円）
/// - `reconstruction_tax` → Integer（復興特別所得税額・円）
/// - `total_tax` → Integer（申告納税額・円・100円未満切り捨て）
/// - `reconstruction_tax_applied?` → true/false
/// - `breakdown` → Array<Hash>（計算内訳）
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::IncomeTax::IncomeTaxResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbIncomeTaxResult {
    base_tax: u64,
    reconstruction_tax: u64,
    total_tax: u64,
    reconstruction_tax_applied: bool,
    breakdown: Vec<IncomeTaxStepData>,
}

impl RbIncomeTaxResult {
    fn base_tax(&self) -> u64 {
        self.base_tax
    }

    fn reconstruction_tax(&self) -> u64 {
        self.reconstruction_tax
    }

    fn total_tax(&self) -> u64 {
        self.total_tax
    }

    fn reconstruction_tax_applied(&self) -> bool {
        self.reconstruction_tax_applied
    }

    /// 計算内訳を Hash の Array で返す。
    ///
    /// 各 Hash のキー:
    /// - `:label`          String
    /// - `:taxable_income` Integer（課税所得金額・円）
    /// - `:rate_numer`     Integer
    /// - `:rate_denom`     Integer
    /// - `:deduction`      Integer（速算表の控除額・円）
    /// - `:result`         Integer（算出税額・円）
    fn breakdown(&self) -> Result<RArray, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let arr = ruby.ary_new();
        for step in &self.breakdown {
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
            self.base_tax,
            self.reconstruction_tax,
            self.total_tax,
            self.reconstruction_tax_applied,
        )
    }
}

// ─── 所得税 Ruby公開関数 ────────────────────────────────────────────────────────

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項 / 復興財源確保法 第13条
///
/// @param taxable_income [Integer] 課税所得金額（円）
/// @param year  [Integer] 対象年度（年）
/// @param month [Integer] 基準日（月）
/// @param day   [Integer] 基準日（日）
/// @param apply_reconstruction_tax [true, false] 復興特別所得税を適用するか
/// @return [JLawRuby::IncomeTax::IncomeTaxResult]
/// @raise [RuntimeError] 対象日に有効な法令パラメータが存在しない場合
fn calc_income_tax(
    taxable_income: u64,
    year: u16,
    month: u8,
    day: u8,
    apply_reconstruction_tax: bool,
) -> Result<RbIncomeTaxResult, Error> {
    let params = load_income_tax_params((year, month, day)).map_err(into_runtime_error)?;

    let mut flags = HashSet::new();
    if apply_reconstruction_tax {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }

    let ctx = IncomeTaxContext {
        taxable_income,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let result = calculate_income_tax(&ctx, &params).map_err(into_runtime_error)?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| IncomeTaxStepData {
            label: step.label.clone(),
            taxable_income: step.taxable_income,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            deduction: step.deduction,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(RbIncomeTaxResult {
        base_tax: result.base_tax.as_yen(),
        reconstruction_tax: result.reconstruction_tax.as_yen(),
        total_tax: result.total_tax.as_yen(),
        reconstruction_tax_applied: result.reconstruction_tax_applied,
        breakdown,
    })
}

// ─── 印紙税 Ruby公開型 ──────────────────────────────────────────────────────────

/// 印紙税の計算結果。
///
/// メソッド:
/// - `tax_amount` → Integer（印紙税額・円）
/// - `bracket_label` → String（適用されたブラケットの表示名）
/// - `reduced_rate_applied?` → true/false
///
/// NOTE: magnus::wrap マクロは展開時に内部で unwrap() を使用するため、
/// Cargo.toml で disallowed_methods = "allow" を設定している
#[magnus::wrap(
    class = "JLawRuby::StampTax::StampTaxResult",
    free_immediately,
    frozen_shareable
)]
pub struct RbStampTaxResult {
    tax_amount: u64,
    bracket_label: String,
    reduced_rate_applied: bool,
}

impl RbStampTaxResult {
    fn tax_amount(&self) -> u64 {
        self.tax_amount
    }

    fn bracket_label(&self) -> String {
        self.bracket_label.clone()
    }

    fn reduced_rate_applied(&self) -> bool {
        self.reduced_rate_applied
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawRuby::StampTax::StampTaxResult tax_amount={} bracket_label={:?} reduced_rate_applied={}>",
            self.tax_amount,
            self.bracket_label,
            self.reduced_rate_applied,
        )
    }
}

// ─── 印紙税 Ruby公開関数 ────────────────────────────────────────────────────────

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条
///
/// @param contract_amount [Integer] 契約金額（円）
/// @param year  [Integer] 契約書作成日（年）
/// @param month [Integer] 契約書作成日（月）
/// @param day   [Integer] 契約書作成日（日）
/// @param is_reduced_rate_applicable [true, false] 軽減税率適用フラグ
///   WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任。
/// @return [JLawRuby::StampTax::StampTaxResult]
/// @raise [RuntimeError] 対象日に有効な法令パラメータが存在しない場合
fn calc_stamp_tax(
    contract_amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate_applicable: bool,
) -> Result<RbStampTaxResult, Error> {
    let params = load_stamp_tax_params((year, month, day)).map_err(into_runtime_error)?;

    let mut flags = HashSet::new();
    if is_reduced_rate_applicable {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }

    let ctx = StampTaxContext {
        contract_amount,
        target_date: (year, month, day),
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    let result = calculate_stamp_tax(&ctx, &params).map_err(into_runtime_error)?;

    Ok(RbStampTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        bracket_label: result.bracket_label,
        reduced_rate_applied: result.reduced_rate_applied,
    })
}

// ─── モジュール定義 ────────────────────────────────────────────────────────────

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let j_law_core = ruby.define_module("JLawRuby")?;
    let real_estate = j_law_core.define_module("RealEstate")?;

    // BrokerageFeeResult クラス
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

    // モジュール関数: JLawRuby::RealEstate.calc_brokerage_fee(...)
    real_estate.define_module_function("calc_brokerage_fee", function!(calc_brokerage_fee, 5))?;

    // ─── 所得税 ───────────────────────────────────────────────────────────────
    let income_tax = j_law_core.define_module("IncomeTax")?;

    // IncomeTaxResult クラス
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

    // モジュール関数: JLawRuby::IncomeTax.calc_income_tax(...)
    income_tax.define_module_function("calc_income_tax", function!(calc_income_tax, 5))?;

    // ─── 印紙税 ───────────────────────────────────────────────────────────────
    let stamp_tax = j_law_core.define_module("StampTax")?;

    // StampTaxResult クラス
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

    // モジュール関数: JLawRuby::StampTax.calc_stamp_tax(...)
    stamp_tax.define_module_function("calc_stamp_tax", function!(calc_stamp_tax, 5))?;

    Ok(())
}
