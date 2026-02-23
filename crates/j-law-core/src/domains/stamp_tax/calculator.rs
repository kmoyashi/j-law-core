use std::collections::HashSet;

use crate::domains::stamp_tax::context::{StampTaxContext, StampTaxFlag};
use crate::domains::stamp_tax::params::StampTaxParams;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::FinalAmount;

/// 印紙税の計算結果。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書（不動産の譲渡に関する契約書）
/// 租税特別措置法 第91条（軽減措置）
#[derive(Debug, Clone)]
pub struct StampTaxResult {
    /// 印紙税額（円）。非課税の場合は0。
    pub tax_amount: FinalAmount,
    /// 適用されたブラケットの表示名。
    pub bracket_label: String,
    /// 軽減税率が適用されたか。
    pub reduced_rate_applied: bool,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<StampTaxFlag>,
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// # 法的根拠
/// 印紙税法 第2条（課税文書の作成者は印紙税を納める義務がある）
/// 印紙税法 別表第一（課税物件表・第1号文書）
/// 租税特別措置法 第91条（不動産の譲渡に関する契約書の印紙税の軽減措置）
///
/// # 計算手順
/// 1. 契約金額が該当するブラケットを特定する
/// 2. 軽減措置の適用条件を判定する（フラグ + 日付範囲 + ブラケットに軽減税額あり）
/// 3. 適用される税額（本則 or 軽減）を返す
pub fn calculate_stamp_tax(
    ctx: &StampTaxContext,
    params: &StampTaxParams,
) -> Result<StampTaxResult, JLawError> {
    let amount = ctx.contract_amount;

    // --- 該当ブラケットの特定 ---
    let bracket = params
        .brackets
        .iter()
        .find(|b| {
            amount >= b.amount_from
                && match b.amount_to_inclusive {
                    Some(to) => amount <= to,
                    None => true,
                }
        })
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "契約金額 {}円 に対応する印紙税額ブラケットが見つかりません",
                amount
            ),
        })?;

    // --- 軽減措置の適用判定 ---
    let date_str = format!(
        "{:04}-{:02}-{:02}",
        ctx.target_date.0, ctx.target_date.1, ctx.target_date.2
    );

    let should_reduce = ctx.policy.should_apply_reduced_rate(
        &date_str,
        params.reduced_rate_from.as_deref(),
        params.reduced_rate_until.as_deref(),
        &ctx.flags,
    );

    let (tax_yen, reduced_applied) = if should_reduce {
        match bracket.reduced_tax_amount {
            Some(reduced) => (reduced, true),
            None => (bracket.tax_amount, false),
        }
    } else {
        (bracket.tax_amount, false)
    };

    Ok(StampTaxResult {
        tax_amount: FinalAmount::new(tax_yen),
        bracket_label: bracket.label.clone(),
        reduced_rate_applied: reduced_applied,
        applied_flags: ctx.flags.clone(),
    })
}
