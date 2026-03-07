use std::collections::HashSet;

use j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    policy::StandardIncomeTaxPolicy,
};
use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::{RealEstateContext, RealEstateFlag},
    policy::StandardMliitPolicy,
};
use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_core::error::JLawError;
use j_law_core::LegalDate;
use j_law_registry::{
    load_brokerage_fee_params, load_consumption_tax_params, load_income_tax_params,
    load_stamp_tax_params,
};

uniffi::setup_scaffolding!();

// ═══════════════════════════════════════════════════════════════════════════════
//  エラー型
// ═══════════════════════════════════════════════════════════════════════════════

/// UniFFI バインディング層のエラー型。
///
/// `JLawError` の3層構造を UniFFI で表現可能な形式に変換する。
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniError {
    /// 法令パラメータ（Registry JSON）の不整合。
    #[error("{message}")]
    RegistryError { message: String },

    /// 呼び出し元の入力不正。
    #[error("{message}")]
    InputError { message: String },

    /// 計算処理中の異常（オーバーフロー等）。
    #[error("{message}")]
    CalculationError { message: String },
}

impl From<JLawError> for UniError {
    fn from(e: JLawError) -> Self {
        match e {
            JLawError::Registry(inner) => UniError::RegistryError {
                message: inner.to_string(),
            },
            JLawError::Input(inner) => UniError::InputError {
                message: inner.to_string(),
            },
            JLawError::Calculation(inner) => UniError::CalculationError {
                message: inner.to_string(),
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  消費税
// ═══════════════════════════════════════════════════════════════════════════════

/// 消費税の計算結果（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniConsumptionTaxResult {
    /// 消費税額（円）。
    pub tax_amount: u64,
    /// 税込金額（円）。
    pub amount_with_tax: u64,
    /// 税抜金額（円）。
    pub amount_without_tax: u64,
    /// 適用税率の分子。
    pub applied_rate_numer: u64,
    /// 適用税率の分母。
    pub applied_rate_denom: u64,
    /// 軽減税率が適用されたか。
    pub is_reduced_rate: bool,
}

/// 消費税法第29条に基づく消費税額を計算する。
///
/// # 法的根拠
/// 消費税法 第29条（税率）
/// 消費税法 第45条（端数処理: 1円未満切り捨て）
///
/// # 計算手順
/// 1. 軽減税率フラグに基づき適用税率を選択する
/// 2. 課税標準額 × 税率（切り捨て）で消費税額を算出する
/// 3. 課税標準額 + 消費税額で税込金額を算出する
#[uniffi::export]
pub fn calc_consumption_tax(
    amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate: bool,
) -> Result<UniConsumptionTaxResult, UniError> {
    let date = LegalDate::new(year, month, day);
    let params = load_consumption_tax_params(date)?;

    let mut flags = HashSet::new();
    if is_reduced_rate {
        flags.insert(ConsumptionTaxFlag::ReducedRate);
    }

    let ctx = ConsumptionTaxContext {
        amount,
        target_date: date,
        flags,
        policy: Box::new(StandardConsumptionTaxPolicy),
    };

    let result = calculate_consumption_tax(&ctx, &params)?;

    Ok(UniConsumptionTaxResult {
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

/// 報酬計算の1ティア分の内訳（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniBreakdownStep {
    /// ティアの表示名（法令上の区分名称）。
    pub label: String,
    /// ティア対象金額（円）。
    pub base_amount: u64,
    /// 適用レートの分子。
    pub rate_numer: u64,
    /// 適用レートの分母。
    pub rate_denom: u64,
    /// ティア計算結果（円・端数切捨て済み）。
    pub result: u64,
}

/// 媒介報酬の計算結果（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniBrokerageFeeResult {
    /// 税抜合計額（円）。
    pub total_without_tax: u64,
    /// 税込合計額（円）。
    pub total_with_tax: u64,
    /// 消費税額（円）。
    pub tax_amount: u64,
    /// 低廉な空き家特例が適用されたか。
    pub low_cost_special_applied: bool,
    /// 各ティアの計算内訳。
    pub breakdown: Vec<UniBreakdownStep>,
}

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行 / 2019年10月1日施行）
///
/// # 計算手順
/// 1. 各ティアの対象金額を求め、個別に切り捨てる
/// 2. 各ティアの結果を合算して税抜き合計を得る
/// 3. 低廉な空き家特例が適用される場合、通常計算が保証額を下回るなら保証額まで引き上げる
/// 4. 消費税ドメインに処理を委譲して税額・税込額を得る
#[uniffi::export]
pub fn calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: bool,
    is_seller: bool,
) -> Result<UniBrokerageFeeResult, UniError> {
    let date = LegalDate::new(year, month, day);
    let params = load_brokerage_fee_params(date)?;

    let mut flags = HashSet::new();
    if is_low_cost_vacant_house {
        flags.insert(RealEstateFlag::IsLowCostVacantHouse);
    }
    if is_seller {
        flags.insert(RealEstateFlag::IsSeller);
    }

    let ctx = RealEstateContext {
        price,
        target_date: date,
        flags,
        policy: Box::new(StandardMliitPolicy),
    };

    let result = calculate_brokerage_fee(&ctx, &params)?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| UniBreakdownStep {
            label: step.label.clone(),
            base_amount: step.base_amount,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(UniBrokerageFeeResult {
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

/// 所得税速算表の1ブラケット分の内訳（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniIncomeTaxStep {
    /// ブラケットの表示名。
    pub label: String,
    /// 課税所得金額（円）。
    pub taxable_income: u64,
    /// 適用税率の分子。
    pub rate_numer: u64,
    /// 適用税率の分母。
    pub rate_denom: u64,
    /// 速算表の控除額（円）。
    pub deduction: u64,
    /// 算出税額（円）。
    pub result: u64,
}

/// 所得税の計算結果（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniIncomeTaxResult {
    /// 基準所得税額（円）。
    pub base_tax: u64,
    /// 復興特別所得税額（円）。
    pub reconstruction_tax: u64,
    /// 申告納税額（円・100円未満切り捨て）。
    pub total_tax: u64,
    /// 復興特別所得税が適用されたか。
    pub reconstruction_tax_applied: bool,
    /// 計算内訳。
    pub breakdown: Vec<UniIncomeTaxStep>,
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項
/// 復興財源確保法 第13条（復興特別所得税: 2013〜2037年）
///
/// # 計算手順
/// 1. 課税所得金額を速算表に適用して基準税額を算出する
/// 2. 復興特別所得税フラグが指定された場合、基準税額 × 2.1% で復興税を算出する
/// 3. 基準税額 + 復興税の合計を100円未満切り捨てして申告納税額を算出する
#[uniffi::export]
pub fn calc_income_tax(
    taxable_income: u64,
    year: u16,
    month: u8,
    day: u8,
    apply_reconstruction_tax: bool,
) -> Result<UniIncomeTaxResult, UniError> {
    let date = LegalDate::new(year, month, day);
    let params = load_income_tax_params(date)?;

    let mut flags = HashSet::new();
    if apply_reconstruction_tax {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }

    let ctx = IncomeTaxContext {
        taxable_income,
        target_date: date,
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    let result = calculate_income_tax(&ctx, &params)?;

    let breakdown = result
        .breakdown
        .iter()
        .map(|step| UniIncomeTaxStep {
            label: step.label.clone(),
            taxable_income: step.taxable_income,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            deduction: step.deduction,
            result: step.result.as_yen(),
        })
        .collect();

    Ok(UniIncomeTaxResult {
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

/// 印紙税の計算結果（UniFFI Record）。
#[derive(Debug, Clone, uniffi::Record)]
pub struct UniStampTaxResult {
    /// 印紙税額（円）。
    pub tax_amount: u64,
    /// 適用されたブラケットの表示名。
    pub bracket_label: String,
    /// 軽減税率が適用されたか。
    pub reduced_rate_applied: bool,
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書（不動産の譲渡に関する契約書）
/// 租税特別措置法 第91条（軽減措置: 2024年3月31日まで）
///
/// # 計算手順
/// 1. 契約金額に対応するブラケットを選択する
/// 2. 軽減税率フラグと適用期間に基づき軽減措置の適否を判定する
/// 3. 適用税額を返す
#[uniffi::export]
pub fn calc_stamp_tax(
    contract_amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate_applicable: bool,
) -> Result<UniStampTaxResult, UniError> {
    let date = LegalDate::new(year, month, day);
    let params = load_stamp_tax_params(date)?;

    let mut flags = HashSet::new();
    if is_reduced_rate_applicable {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }

    let ctx = StampTaxContext {
        contract_amount,
        target_date: date,
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    let result = calculate_stamp_tax(&ctx, &params)?;

    Ok(UniStampTaxResult {
        tax_amount: result.tax_amount.as_yen(),
        bracket_label: result.bracket_label,
        reduced_rate_applied: result.reduced_rate_applied,
    })
}
