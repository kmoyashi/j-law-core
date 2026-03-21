use crate::domains::stamp_tax::context::{StampTaxContext, StampTaxFlag};
use crate::domains::stamp_tax::params::{
    StampTaxAmountUsage, StampTaxBracket, StampTaxChargeMode, StampTaxDocumentParams,
    StampTaxParams, StampTaxSpecialRule,
};
use crate::error::{CalculationError, InputError, JLawError};
use crate::types::amount::FinalAmount;

/// 印紙税の計算内訳。
#[derive(Debug, Clone)]
pub struct StampTaxBreakdownStep {
    pub rule_code: String,
    pub label: String,
    pub tax_amount: FinalAmount,
}

/// 印紙税の計算結果。
///
/// # 法的根拠
/// 印紙税法 別表第一 / 租税特別措置法 第91条
#[derive(Debug, Clone)]
pub struct StampTaxResult {
    pub tax_amount: FinalAmount,
    pub rule_label: String,
    pub applied_special_rule: Option<String>,
    pub breakdown: Vec<StampTaxBreakdownStep>,
}

struct ResolvedTaxRule {
    tax_amount: u64,
    rule_label: String,
}

/// 印紙税法 別表第一に基づく印紙税額を計算する。
pub fn calculate_stamp_tax(
    ctx: &StampTaxContext,
    params: &StampTaxParams,
) -> Result<StampTaxResult, JLawError> {
    ctx.target_date.validate()?;

    let document = params.document_params(ctx.document_code).ok_or_else(|| {
        InputError::InvalidStampTaxInput {
            field: "document_code".into(),
            reason: format!(
                "対象文書コードのパラメータが存在しません: {}",
                ctx.document_code
            ),
        }
    })?;

    validate_context(ctx, document)?;

    let special_rule = ctx.policy.select_special_rule(ctx, document);
    let (resolved, applied_special_rule, breakdown_rule_code) = match special_rule {
        Some(rule) => (
            resolve_special_rule(ctx, document, rule)?,
            Some(rule.code.clone()),
            rule.code.clone(),
        ),
        None => (resolve_base_rule(ctx, document)?, None, "base".into()),
    };

    Ok(StampTaxResult {
        tax_amount: FinalAmount::new(resolved.tax_amount),
        rule_label: resolved.rule_label.clone(),
        applied_special_rule,
        breakdown: vec![StampTaxBreakdownStep {
            rule_code: breakdown_rule_code,
            label: resolved.rule_label,
            tax_amount: FinalAmount::new(resolved.tax_amount),
        }],
    })
}

fn validate_context(
    ctx: &StampTaxContext,
    document: &StampTaxDocumentParams,
) -> Result<(), JLawError> {
    match document.amount_usage {
        StampTaxAmountUsage::Required if ctx.stated_amount.is_none() => {
            return Err(InputError::InvalidStampTaxInput {
                field: "stated_amount".into(),
                reason: format!("{} には記載金額の指定が必要です", ctx.document_code),
            }
            .into());
        }
        StampTaxAmountUsage::Unsupported if ctx.stated_amount.is_some() => {
            return Err(InputError::InvalidStampTaxInput {
                field: "stated_amount".into(),
                reason: format!("{} では記載金額を指定できません", ctx.document_code),
            }
            .into());
        }
        _ => {}
    }

    for flag in &ctx.flags {
        validate_flag_for_document(*flag, ctx.document_code, ctx.stated_amount)?;
    }

    Ok(())
}

fn validate_flag_for_document(
    flag: StampTaxFlag,
    document_code: crate::domains::stamp_tax::context::StampTaxDocumentCode,
    stated_amount: Option<u64>,
) -> Result<(), JLawError> {
    if !flag.allowed_document_codes().contains(&document_code) {
        return Err(InputError::InvalidStampTaxInput {
            field: "flags".into(),
            reason: format!("{flag} は {document_code} には指定できません"),
        }
        .into());
    }

    if flag.requires_stated_amount() && stated_amount.is_none() {
        return Err(InputError::InvalidStampTaxInput {
            field: "stated_amount".into(),
            reason: format!("{flag} を指定する場合は記載金額が必要です"),
        }
        .into());
    }

    Ok(())
}

fn resolve_special_rule(
    ctx: &StampTaxContext,
    document: &StampTaxDocumentParams,
    rule: &StampTaxSpecialRule,
) -> Result<ResolvedTaxRule, JLawError> {
    if let Some(tax_amount) = rule.tax_amount {
        let label = rule
            .rule_label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| rule.label.clone());
        return Ok(ResolvedTaxRule {
            tax_amount,
            rule_label: label,
        });
    }

    if !rule.brackets.is_empty() || rule.no_amount_tax_amount.is_some() {
        return resolve_amount_table(
            ctx.stated_amount,
            &rule.brackets,
            rule.no_amount_tax_amount,
            rule.no_amount_rule_label.as_deref(),
            document,
        );
    }

    Err(CalculationError::PolicyNotApplicable {
        reason: format!("特例ルールの税額定義が不正です: {}", rule.code),
    }
    .into())
}

fn resolve_base_rule(
    ctx: &StampTaxContext,
    document: &StampTaxDocumentParams,
) -> Result<ResolvedTaxRule, JLawError> {
    match document.charge_mode {
        StampTaxChargeMode::AmountBrackets => resolve_amount_table(
            ctx.stated_amount,
            &document.brackets,
            document.no_amount_tax_amount,
            document.no_amount_rule_label.as_deref(),
            document,
        ),
        StampTaxChargeMode::FixedPerDocument | StampTaxChargeMode::FixedPerYear => {
            let tax_amount =
                document
                    .base_tax_amount
                    .ok_or_else(|| CalculationError::PolicyNotApplicable {
                        reason: format!("固定税額が未設定です: {}", document.code),
                    })?;
            Ok(ResolvedTaxRule {
                tax_amount,
                rule_label: document.base_rule_label.clone(),
            })
        }
    }
}

fn resolve_amount_table(
    stated_amount: Option<u64>,
    brackets: &[StampTaxBracket],
    no_amount_tax_amount: Option<u64>,
    no_amount_rule_label: Option<&str>,
    document: &StampTaxDocumentParams,
) -> Result<ResolvedTaxRule, JLawError> {
    match stated_amount {
        Some(amount) => {
            let bracket = brackets
                .iter()
                .find(|bracket| bracket.matches(amount))
                .ok_or_else(|| CalculationError::PolicyNotApplicable {
                    reason: format!(
                        "{} の記載金額 {}円 に対応する税額区分が見つかりません",
                        document.code, amount
                    ),
                })?;
            Ok(ResolvedTaxRule {
                tax_amount: bracket.tax_amount,
                rule_label: bracket.label.clone(),
            })
        }
        None => {
            let tax_amount =
                no_amount_tax_amount.ok_or_else(|| InputError::InvalidStampTaxInput {
                    field: "stated_amount".into(),
                    reason: format!("{} には記載金額の指定が必要です", document.code),
                })?;
            let label = no_amount_rule_label
                .unwrap_or("記載金額の記載のないもの")
                .to_string();
            Ok(ResolvedTaxRule {
                tax_amount,
                rule_label: label,
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use super::*;
    use crate::domains::stamp_tax::context::StampTaxDocumentCode;
    use crate::domains::stamp_tax::params::{StampTaxCitation, StampTaxSpecialRule};
    use crate::domains::stamp_tax::policy::StandardNtaPolicy;
    use crate::types::date::LegalDate;

    fn amount_document() -> StampTaxDocumentParams {
        StampTaxDocumentParams {
            code: StampTaxDocumentCode::Article17SalesReceipt,
            label: "売上代金受取書".into(),
            citation: StampTaxCitation {
                law_name: "印紙税法".into(),
                article: "別表第一 第17号文書".into(),
            },
            charge_mode: StampTaxChargeMode::AmountBrackets,
            amount_usage: StampTaxAmountUsage::Optional,
            base_rule_label: "通常".into(),
            base_tax_amount: None,
            brackets: vec![StampTaxBracket {
                label: "100万円以下のもの".into(),
                amount_from: 50_000,
                amount_to_inclusive: Some(1_000_000),
                tax_amount: 200,
            }],
            no_amount_tax_amount: Some(200),
            no_amount_rule_label: Some("受取金額の記載のないもの".into()),
            special_rules: vec![StampTaxSpecialRule {
                code: "article17_non_business_exempt".into(),
                label: "非課税".into(),
                priority: 1,
                effective_from: None,
                effective_until: None,
                required_flags: vec![StampTaxFlag::Article17NonBusinessExempt],
                tax_amount: Some(0),
                rule_label: Some("営業に関しないもの".into()),
                brackets: vec![],
                no_amount_tax_amount: None,
                no_amount_rule_label: None,
            }],
        }
    }

    #[test]
    fn calculates_base_amount_rule() {
        let mut documents = BTreeMap::new();
        documents.insert(
            StampTaxDocumentCode::Article17SalesReceipt,
            amount_document(),
        );
        let params = StampTaxParams { documents };
        let ctx = StampTaxContext {
            document_code: StampTaxDocumentCode::Article17SalesReceipt,
            stated_amount: Some(60_000),
            target_date: LegalDate::new(2024, 8, 1),
            flags: HashSet::new(),
            policy: Box::new(StandardNtaPolicy),
        };

        let result = calculate_stamp_tax(&ctx, &params).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 200);
        assert_eq!(result.rule_label, "100万円以下のもの");
    }

    #[test]
    fn applies_special_rule() {
        let mut documents = BTreeMap::new();
        documents.insert(
            StampTaxDocumentCode::Article17SalesReceipt,
            amount_document(),
        );
        let params = StampTaxParams { documents };
        let mut flags = HashSet::new();
        flags.insert(StampTaxFlag::Article17NonBusinessExempt);
        let ctx = StampTaxContext {
            document_code: StampTaxDocumentCode::Article17SalesReceipt,
            stated_amount: Some(60_000),
            target_date: LegalDate::new(2024, 8, 1),
            flags,
            policy: Box::new(StandardNtaPolicy),
        };

        let result = calculate_stamp_tax(&ctx, &params).unwrap();
        assert_eq!(result.tax_amount.as_yen(), 0);
        assert_eq!(
            result.applied_special_rule.as_deref(),
            Some("article17_non_business_exempt")
        );
    }
}
