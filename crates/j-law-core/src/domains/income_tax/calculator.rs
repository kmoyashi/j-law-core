use std::collections::HashSet;

use crate::domains::income_tax::context::{IncomeTaxContext, IncomeTaxFlag};
use crate::domains::income_tax::params::{
    DefenseTaxParams, IncomeTaxParams, ReconstructionTaxParams,
};
use crate::domains::income_tax::policy::IncomeTaxPolicy;
use crate::error::{CalculationError, JLawError};
use crate::types::amount::{FinalAmount, IntermediateAmount};
use crate::types::date::LegalDate;
use crate::types::rate::{MultiplyOrder, Rate};
use crate::types::rounding::RoundingStrategy;

/// 所得税の計算ステップ（内訳明細用）。
///
/// 速算表の適用結果を記録する。
#[derive(Debug, Clone)]
pub struct IncomeTaxStep {
    /// ブラケットの表示名（例: "330万円超695万円以下"）。
    pub label: String,
    /// 課税所得金額（円）。
    pub taxable_income: u64,
    /// 適用された税率の分子。
    pub rate_numer: u64,
    /// 適用された税率の分母。
    pub rate_denom: u64,
    /// 速算表の控除額（円）。
    pub deduction: u64,
    /// このステップでの算出税額（課税所得 × 税率 - 控除額）。
    pub result: FinalAmount,
}

/// 所得税の計算結果。
///
/// # 法的根拠
/// 所得税法 第89条第1項（所得税の税率）
/// 復興財源確保法 第13条（復興特別所得税）
/// 防衛財源確保法 第5条の9（防衛特別所得税）
/// 防衛財源確保法 第5条の22第2項 / 国税通則法 第119条第1項（税額の端数処理）
#[derive(Debug, Clone)]
pub struct IncomeTaxResult {
    /// 基準所得税額（付加税を含まない）。
    pub base_tax: FinalAmount,
    /// 復興特別所得税額（適用されない場合は0円）。
    ///
    /// 防衛特別所得税と併課される年分（2027年〜2047年分）は、付加税の合計額
    /// （基準所得税額 × 合計税率、1円未満切り捨て）から防衛特別所得税額を差し引いた額。
    pub reconstruction_tax: FinalAmount,
    /// 防衛特別所得税額（適用されない場合は0円）。
    ///
    /// 基準所得税額 × 1%（1円未満切り捨て）。
    pub defense_tax: FinalAmount,
    /// 申告納税額（基準所得税額 + 復興特別所得税額 + 防衛特別所得税額）。
    ///
    /// 合計額の100円未満切り捨て（防衛財源確保法 第5条の22第2項 / 国税通則法 第119条第1項）。
    pub total_tax: FinalAmount,
    /// 計算の内訳ステップ。
    pub breakdown: Vec<IncomeTaxStep>,
    /// 適用されたフラグ。
    pub applied_flags: HashSet<IncomeTaxFlag>,
    /// 復興特別所得税が適用されたか。
    pub reconstruction_tax_applied: bool,
    /// 防衛特別所得税が適用されたか。
    pub defense_tax_applied: bool,
}

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項（税率表・速算表方式）
/// 復興財源確保法 第13条（復興特別所得税: 2026年分まで 2.1%、2027〜2047年分 1.1%）
/// 防衛財源確保法 第5条の9（防衛特別所得税: 2027年分以後 1%）
/// 防衛財源確保法 第5条の22第2項 / 国税通則法 第119条第1項（申告納税額の100円未満切り捨て）
///
/// # 計算手順
/// 1. 課税所得金額が該当するブラケットを特定する
/// 2. 速算表方式で基準所得税額を算出: 課税所得金額 × 税率 - 控除額
/// 3. 付加税を適用する場合（[`IncomeTaxFlag::ApplyReconstructionTax`]）、その年分に課される
///    付加税の合計額 = 基準所得税額 × 合計税率（1円未満切り捨て）
/// 4. 防衛特別所得税額 = 基準所得税額 × 1%（1円未満切り捨て）、
///    復興特別所得税額 = 付加税の合計額 - 防衛特別所得税額
/// 5. 申告納税額 = 基準所得税額 + 付加税の合計額（100円未満切り捨て）
///
/// 確定金額の端数計算は所得税・復興特別所得税・防衛特別所得税の合計額で行う
/// （防衛財源確保法 第5条の22第2項）ため、付加税は合計額で端数処理してから内訳に按分する。
pub fn calculate_income_tax(
    ctx: &IncomeTaxContext,
    params: &IncomeTaxParams,
) -> Result<IncomeTaxResult, JLawError> {
    calculate_income_tax_inner(
        ctx.taxable_income,
        ctx.target_date,
        &ctx.flags,
        ctx.policy.as_ref(),
        params,
    )
}

pub(crate) fn calculate_income_tax_inner(
    taxable_income: u64,
    target_date: LegalDate,
    flags: &HashSet<IncomeTaxFlag>,
    policy: &dyn IncomeTaxPolicy,
    params: &IncomeTaxParams,
) -> Result<IncomeTaxResult, JLawError> {
    target_date.validate()?;

    let income = taxable_income;
    let tax_rounding = policy.tax_rounding();

    // --- 課税所得金額が0の場合 ---
    if income == 0 {
        return Ok(IncomeTaxResult {
            base_tax: FinalAmount::new(0),
            reconstruction_tax: FinalAmount::new(0),
            defense_tax: FinalAmount::new(0),
            total_tax: FinalAmount::new(0),
            breakdown: vec![],
            applied_flags: flags.clone(),
            reconstruction_tax_applied: false,
            defense_tax_applied: false,
        });
    }

    // --- 該当ブラケットの特定 ---
    let bracket = params
        .brackets
        .iter()
        .find(|b| {
            income >= b.income_from
                && match b.income_to_inclusive {
                    Some(to) => income <= to,
                    None => true,
                }
        })
        .ok_or_else(|| CalculationError::PolicyNotApplicable {
            reason: format!(
                "課税所得金額 {}円 に対応する税率ブラケットが見つかりません",
                income
            ),
        })?;

    // --- 速算表方式による基準所得税額の計算 ---
    // 税額 = 課税所得金額 × 税率 - 控除額
    let rate = Rate::new(bracket.rate_numer, bracket.rate_denom).map_err(|_| {
        CalculationError::Overflow {
            step: "income_tax_rate".into(),
        }
    })?;
    let gross_tax = rate
        .apply(
            &IntermediateAmount::from_exact(income),
            MultiplyOrder::MultiplyFirst,
            tax_rounding,
        )?
        .finalize(tax_rounding)?;

    let base_tax_yen = gross_tax
        .as_yen()
        .checked_sub(bracket.deduction)
        .ok_or_else(|| CalculationError::Overflow {
            step: "base_tax_deduction".into(),
        })?;

    let base_tax = FinalAmount::new(base_tax_yen);

    let breakdown = vec![IncomeTaxStep {
        label: bracket.label.clone(),
        taxable_income: income,
        rate_numer: bracket.rate_numer,
        rate_denom: bracket.rate_denom,
        deduction: bracket.deduction,
        result: base_tax,
    }];

    // --- 付加税（復興特別所得税・防衛特別所得税） ---
    let special = if policy.should_apply_reconstruction_tax(target_date.year, flags) {
        calculate_special_income_taxes(
            base_tax_yen,
            target_date.year,
            params,
            policy.reconstruction_tax_rounding(),
        )?
    } else {
        SpecialIncomeTaxes::default()
    };

    // --- 申告納税額（合計額の100円未満切り捨て） ---
    // 防衛財源確保法 第5条の22第2項 / 国税通則法 第119条第1項
    let total_before_truncation =
        base_tax_yen
            .checked_add(special.total_yen)
            .ok_or_else(|| CalculationError::Overflow {
                step: "total_tax".into(),
            })?;
    let total_tax = FinalAmount::new(truncate_below_100(total_before_truncation));

    Ok(IncomeTaxResult {
        base_tax,
        reconstruction_tax: FinalAmount::new(special.reconstruction_yen),
        defense_tax: FinalAmount::new(special.defense_yen),
        total_tax,
        breakdown,
        applied_flags: flags.clone(),
        reconstruction_tax_applied: special.reconstruction_applied,
        defense_tax_applied: special.defense_applied,
    })
}

/// その年分に課される付加税の計算結果。
#[derive(Debug, Default, PartialEq, Eq)]
struct SpecialIncomeTaxes {
    /// 付加税の合計額（1円未満切り捨て）。
    total_yen: u64,
    reconstruction_yen: u64,
    defense_yen: u64,
    reconstruction_applied: bool,
    defense_applied: bool,
}

/// 基準所得税額に対する付加税（復興特別所得税・防衛特別所得税）を計算する。
///
/// 課税期間はパラメータで判定し、期間外の付加税は課さない。
/// 合計額を先に1円未満切り捨てで求め、防衛特別所得税額（1円未満切り捨て）を差し引いた
/// 残額を復興特別所得税額とする。これにより
/// `基準所得税額 + 復興特別所得税額 + 防衛特別所得税額` が端数処理前の申告納税額と一致する。
fn calculate_special_income_taxes(
    base_tax_yen: u64,
    year: u16,
    params: &IncomeTaxParams,
    rounding: RoundingStrategy,
) -> Result<SpecialIncomeTaxes, JLawError> {
    let reconstruction = params
        .reconstruction_tax
        .as_ref()
        .filter(|rt| rt.applies_to_year(year));
    let defense = params
        .defense_tax
        .as_ref()
        .filter(|dt| dt.applies_to_year(year));

    let (numer, denom) = match (reconstruction, defense) {
        (None, None) => return Ok(SpecialIncomeTaxes::default()),
        (Some(rt), None) => (rt.rate_numer, rt.rate_denom),
        (None, Some(dt)) => (dt.rate_numer, dt.rate_denom),
        (Some(rt), Some(dt)) => add_fractions(rt, dt)?,
    };
    let total_yen = apply_rate(base_tax_yen, numer, denom, rounding, "special_income_tax")?;
    let defense_yen = match defense {
        Some(dt) => apply_rate(
            base_tax_yen,
            dt.rate_numer,
            dt.rate_denom,
            rounding,
            "defense_tax",
        )?,
        None => 0,
    };
    let reconstruction_yen =
        total_yen
            .checked_sub(defense_yen)
            .ok_or_else(|| CalculationError::Overflow {
                step: "reconstruction_tax".into(),
            })?;

    Ok(SpecialIncomeTaxes {
        total_yen,
        reconstruction_yen,
        defense_yen,
        reconstruction_applied: reconstruction.is_some(),
        defense_applied: defense.is_some(),
    })
}

/// 復興特別所得税率と防衛特別所得税率の合計税率を分数で返す。
fn add_fractions(
    rt: &ReconstructionTaxParams,
    dt: &DefenseTaxParams,
) -> Result<(u64, u64), JLawError> {
    let overflow = || CalculationError::Overflow {
        step: "special_income_tax_rate".into(),
    };
    let numer = rt
        .rate_numer
        .checked_mul(dt.rate_denom)
        .and_then(|a| {
            dt.rate_numer
                .checked_mul(rt.rate_denom)
                .and_then(|b| a.checked_add(b))
        })
        .ok_or_else(overflow)?;
    let denom = rt
        .rate_denom
        .checked_mul(dt.rate_denom)
        .ok_or_else(overflow)?;
    Ok((numer, denom))
}

fn apply_rate(
    amount_yen: u64,
    numer: u64,
    denom: u64,
    rounding: RoundingStrategy,
    step: &str,
) -> Result<u64, JLawError> {
    let rate = Rate::new(numer, denom).map_err(|_| CalculationError::Overflow {
        step: format!("{step}_rate"),
    })?;
    Ok(rate
        .apply(
            &IntermediateAmount::from_exact(amount_yen),
            MultiplyOrder::MultiplyFirst,
            rounding,
        )?
        .finalize(rounding)?
        .as_yen())
}

/// 100円未満を切り捨てる（国税通則法 第119条第1項）。
fn truncate_below_100(amount: u64) -> u64 {
    amount / 100 * 100
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn truncate_below_100_works() {
        assert_eq!(truncate_below_100(0), 0);
        assert_eq!(truncate_below_100(99), 0);
        assert_eq!(truncate_below_100(100), 100);
        assert_eq!(truncate_below_100(199), 100);
        assert_eq!(truncate_below_100(584_522), 584_500);
    }

    fn params(reconstruction: Option<(u64, u64)>, defense: Option<(u64, u64)>) -> IncomeTaxParams {
        IncomeTaxParams {
            brackets: vec![],
            reconstruction_tax: reconstruction.map(|(n, d)| ReconstructionTaxParams {
                rate_numer: n,
                rate_denom: d,
                effective_from_year: 2013,
                effective_to_year_inclusive: 2047,
            }),
            defense_tax: defense.map(|(n, d)| DefenseTaxParams {
                rate_numer: n,
                rate_denom: d,
                effective_from_year: 2027,
            }),
        }
    }

    fn special(base: u64, year: u16, params: &IncomeTaxParams) -> SpecialIncomeTaxes {
        calculate_special_income_taxes(base, year, params, RoundingStrategy::Floor).unwrap()
    }

    #[test]
    fn reconstruction_only_until_2026() {
        let result = special(50_000, 2024, &params(Some((21, 1000)), None));
        assert_eq!(result.total_yen, 1_050);
        assert_eq!(result.reconstruction_yen, 1_050);
        assert_eq!(result.defense_yen, 0);
        assert!(result.reconstruction_applied);
        assert!(!result.defense_applied);
    }

    #[test]
    fn reconstruction_and_defense_from_2027() {
        let result = special(50_000, 2027, &params(Some((11, 1000)), Some((1, 100))));
        assert_eq!(result.total_yen, 1_050);
        assert_eq!(result.defense_yen, 500);
        assert_eq!(result.reconstruction_yen, 550);
        assert!(result.reconstruction_applied);
        assert!(result.defense_applied);
    }

    #[test]
    fn combined_special_tax_is_rounded_once() {
        // 99,999 × 1% = 999.99、× 1.1% = 1,099.989、× 2.1% = 2,099.979
        // 個別に切り捨てると 2,098 円になるが、合計額で端数処理して 2,099 円とする。
        let result = special(99_999, 2027, &params(Some((11, 1000)), Some((1, 100))));
        assert_eq!(result.total_yen, 2_099);
        assert_eq!(result.defense_yen, 999);
        assert_eq!(result.reconstruction_yen, 1_100);
    }

    #[test]
    fn defense_only_from_2048() {
        let result = special(50_000, 2048, &params(None, Some((1, 100))));
        assert_eq!(result.total_yen, 500);
        assert_eq!(result.defense_yen, 500);
        assert_eq!(result.reconstruction_yen, 0);
        assert!(!result.reconstruction_applied);
        assert!(result.defense_applied);
    }

    #[test]
    fn special_taxes_respect_param_periods() {
        let both = params(Some((11, 1000)), Some((1, 100)));
        assert_eq!(special(50_000, 2012, &both), SpecialIncomeTaxes::default());
        // 2048年分は復興特別所得税の課税期間外。
        let result = special(50_000, 2048, &both);
        assert!(!result.reconstruction_applied);
        assert_eq!(result.total_yen, 500);
        // 2026年分は防衛特別所得税の課税期間外。
        let result = special(50_000, 2026, &both);
        assert!(!result.defense_applied);
        assert_eq!(result.total_yen, 550);
    }

    #[test]
    fn total_with_defense_matches_former_reconstruction_rate() {
        // 合計税率は 2.1% のまま変わらないため、付加税の合計額は改正前と一致する。
        let before = params(Some((21, 1000)), None);
        let after = params(Some((11, 1000)), Some((1, 100)));
        for base in (0..300_000).step_by(7) {
            let old = special(base, 2026, &before);
            let new = special(base, 2027, &after);
            assert_eq!(old.total_yen, new.total_yen, "基準所得税額 {base}円");
            assert_eq!(
                new.reconstruction_yen + new.defense_yen,
                new.total_yen,
                "基準所得税額 {base}円"
            );
        }
    }
}
