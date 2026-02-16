use std::collections::HashSet;

use magnus::{function, method, Error, Module, RArray, RHash, Ruby, Symbol};

use ::j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
    RealEstateFlag,
};
use ::j_law_registry::load_brokerage_fee_params;

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
#[magnus::wrap(class = "JLawCore::RealEstate::BrokerageFeeResult", free_immediately, frozen_shareable)]
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
    fn breakdown(&self) -> RArray {
        // SAFETY: Magnus が #[magnus::wrap] で wrap したオブジェクトのメソッドは
        // Ruby の GIL 保持下で呼ばれるため、Ruby ランタイムは必ず初期化済みである。
        let ruby = unsafe { Ruby::get_unchecked() };
        let arr = ruby.ary_new();
        for step in &self.breakdown {
            let hash = ruby.hash_new();
            hash.aset(Symbol::new("label"), step.label.as_str()).unwrap();
            hash.aset(Symbol::new("base_amount"), step.base_amount).unwrap();
            hash.aset(Symbol::new("rate_numer"), step.rate_numer).unwrap();
            hash.aset(Symbol::new("rate_denom"), step.rate_denom).unwrap();
            hash.aset(Symbol::new("result"), step.result).unwrap();
            arr.push(hash).unwrap();
        }
        arr
    }

    fn inspect(&self) -> String {
        format!(
            "#<JLawCore::RealEstate::BrokerageFeeResult total_without_tax={} total_with_tax={} tax_amount={} low_cost_special_applied={}>",
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
/// @return [JLawCore::RealEstate::BrokerageFeeResult]
/// @raise [RuntimeError] 対象日に有効な法令パラメータが存在しない場合
fn calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
) -> Result<RbBrokerageFeeResult, Error> {
    let params =
        load_brokerage_fee_params((year, month, day)).map_err(into_runtime_error)?;

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

    let result =
        calculate_brokerage_fee(&ctx, &params).map_err(into_runtime_error)?;

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

// ─── モジュール定義 ────────────────────────────────────────────────────────────

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let j_law_core = ruby.define_module("JLawCore")?;
    let real_estate = j_law_core.define_module("RealEstate")?;

    // BrokerageFeeResult クラス
    let result_class =
        real_estate.define_class("BrokerageFeeResult", ruby.class_object())?;
    result_class.define_method(
        "total_without_tax",
        method!(RbBrokerageFeeResult::total_without_tax, 0),
    )?;
    result_class.define_method(
        "total_with_tax",
        method!(RbBrokerageFeeResult::total_with_tax, 0),
    )?;
    result_class.define_method(
        "tax_amount",
        method!(RbBrokerageFeeResult::tax_amount, 0),
    )?;
    result_class.define_method(
        "low_cost_special_applied?",
        method!(RbBrokerageFeeResult::low_cost_special_applied, 0),
    )?;
    result_class.define_method(
        "breakdown",
        method!(RbBrokerageFeeResult::breakdown, 0),
    )?;
    result_class.define_method(
        "inspect",
        method!(RbBrokerageFeeResult::inspect, 0),
    )?;
    result_class.define_method(
        "to_s",
        method!(RbBrokerageFeeResult::inspect, 0),
    )?;

    // モジュール関数: JLawCore::RealEstate.calc_brokerage_fee(...)
    real_estate.define_module_function(
        "calc_brokerage_fee",
        function!(calc_brokerage_fee, 5),
    )?;

    Ok(())
}
