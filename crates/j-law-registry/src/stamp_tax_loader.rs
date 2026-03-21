use std::collections::BTreeMap;
use std::str::FromStr;

use crate::stamp_tax_schema::{
    StampTaxBracketEntry, StampTaxDocumentParamsEntry, StampTaxHistoryEntry, StampTaxRegistry,
    StampTaxSpecialRuleEntry,
};
use j_law_core::domains::stamp_tax::context::{StampTaxDocumentCode, StampTaxFlag};
use j_law_core::domains::stamp_tax::params::{
    StampTaxAmountUsage, StampTaxBracket, StampTaxChargeMode, StampTaxCitation,
    StampTaxDocumentParams, StampTaxParams, StampTaxSpecialRule,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

const PATH: &str = "stamp_tax/stamp_tax.json";

/// `stamp_tax.json` をロードして `target_date` に対応するパラメータを返す。
pub fn load_stamp_tax_params(target_date: LegalDate) -> Result<StampTaxParams, JLawError> {
    target_date.validate()?;

    let json_str = include_str!("../data/stamp_tax/stamp_tax.json");

    let registry: StampTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: PATH.into(),
            cause: e.to_string(),
        })?;
    validate_registry(&registry)?;

    let date_str = target_date.to_date_str();

    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    to_params(entry)
}

/// `StampTaxRegistry` の整合性を検証する。
///
/// # 検証内容
/// - 適用期間の重複（Overlap）
/// - 適用期間の空白（Gap）
fn validate_registry(registry: &StampTaxRegistry) -> Result<(), RegistryError> {
    let domain = &registry.domain;

    // 期間の重複・ギャップチェック
    let mut sorted = registry.history.clone();
    sorted.sort_by(|a, b| a.effective_from.cmp(&b.effective_from));

    for [current, next] in sorted.array_windows::<2>() {
        let current_until = match &current.effective_until {
            Some(d) => d.clone(),
            None => {
                return Err(RegistryError::PeriodOverlap {
                    domain: domain.clone(),
                    from: next.effective_from.clone(),
                    until: "open-ended".into(),
                });
            }
        };

        if current_until >= next.effective_from {
            return Err(RegistryError::PeriodOverlap {
                domain: domain.clone(),
                from: next.effective_from.clone(),
                until: current_until.clone(),
            });
        }

        let until_date = LegalDate::from_date_str(&current_until).ok_or_else(|| {
            RegistryError::InvalidDateFormat {
                domain: domain.clone(),
                value: current_until.clone(),
            }
        })?;
        let expected_next_from = until_date.next_day().to_date_str();
        if expected_next_from != next.effective_from {
            return Err(RegistryError::PeriodGap {
                domain: domain.clone(),
                end: current_until,
                next_start: next.effective_from.clone(),
            });
        }
    }

    Ok(())
}

fn find_entry<'a>(
    registry: &'a StampTaxRegistry,
    date_str: &str,
) -> Option<&'a StampTaxHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &StampTaxHistoryEntry) -> Result<StampTaxParams, JLawError> {
    let mut documents = BTreeMap::new();

    for (key, value) in &entry.params.documents {
        let document_code = StampTaxDocumentCode::from_str(key).map_err(registry_parse_error)?;
        let document_params = to_document_params(document_code, value)?;
        documents.insert(document_code, document_params);
    }

    if documents.is_empty() {
        return Err(registry_parse_error(InputError::InvalidStampTaxInput {
            field: "documents".into(),
            reason: "印紙税文書コードが1件も定義されていません".into(),
        }));
    }

    Ok(StampTaxParams { documents })
}

fn to_document_params(
    code: StampTaxDocumentCode,
    entry: &StampTaxDocumentParamsEntry,
) -> Result<StampTaxDocumentParams, JLawError> {
    let charge_mode =
        StampTaxChargeMode::from_str(&entry.charge_mode).map_err(registry_parse_error)?;
    let amount_usage =
        StampTaxAmountUsage::from_str(&entry.amount_usage).map_err(registry_parse_error)?;
    let brackets = to_brackets(&entry.brackets)?;
    validate_brackets(code, &brackets)?;

    let special_rules = entry
        .special_rules
        .iter()
        .map(|rule| to_special_rule(code, rule))
        .collect::<Result<Vec<_>, _>>()?;

    match charge_mode {
        StampTaxChargeMode::AmountBrackets => {
            if brackets.is_empty() {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: code.to_string(),
                    reason: "amount_brackets には brackets が必要です".into(),
                }));
            }
            if entry.base_tax_amount.is_some() {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: code.to_string(),
                    reason: "amount_brackets に base_tax_amount は指定できません".into(),
                }));
            }
        }
        StampTaxChargeMode::FixedPerDocument | StampTaxChargeMode::FixedPerYear => {
            if entry.base_tax_amount.is_none() {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: code.to_string(),
                    reason: "固定税額文書には base_tax_amount が必要です".into(),
                }));
            }
            if !brackets.is_empty() {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: code.to_string(),
                    reason: "固定税額文書の base brackets は空である必要があります".into(),
                }));
            }
        }
    }

    if matches!(amount_usage, StampTaxAmountUsage::Unsupported)
        && entry.no_amount_tax_amount.is_some()
    {
        return Err(registry_parse_error(InputError::InvalidStampTaxInput {
            field: code.to_string(),
            reason: "amount_usage=unsupported では no_amount_tax_amount を指定できません".into(),
        }));
    }

    Ok(StampTaxDocumentParams {
        code,
        label: entry.label.clone(),
        citation: StampTaxCitation {
            law_name: entry.citation.law_name.clone(),
            article: entry.citation.article.clone(),
        },
        charge_mode,
        amount_usage,
        base_rule_label: entry.base_rule_label.clone(),
        base_tax_amount: entry.base_tax_amount,
        brackets,
        no_amount_tax_amount: entry.no_amount_tax_amount,
        no_amount_rule_label: entry.no_amount_rule_label.clone(),
        special_rules,
    })
}

fn to_special_rule(
    document_code: StampTaxDocumentCode,
    entry: &StampTaxSpecialRuleEntry,
) -> Result<StampTaxSpecialRule, JLawError> {
    let required_flags = entry
        .required_flags
        .iter()
        .map(|flag| StampTaxFlag::from_str(flag).map_err(registry_parse_error))
        .collect::<Result<Vec<_>, _>>()?;

    for flag in &required_flags {
        if !flag.allowed_document_codes().contains(&document_code) {
            return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                field: document_code.to_string(),
                reason: format!(
                    "特例ルール {} で許可されない flag {} を参照しています",
                    entry.code, flag
                ),
            }));
        }
    }

    let brackets = to_brackets(&entry.brackets)?;
    if !brackets.is_empty() {
        validate_brackets(document_code, &brackets)?;
    }

    Ok(StampTaxSpecialRule {
        code: entry.code.clone(),
        label: entry.label.clone(),
        priority: entry.priority,
        effective_from: entry.effective_from.clone(),
        effective_until: entry.effective_until.clone(),
        required_flags,
        tax_amount: entry.tax_amount,
        rule_label: entry.rule_label.clone(),
        brackets,
        no_amount_tax_amount: entry.no_amount_tax_amount,
        no_amount_rule_label: entry.no_amount_rule_label.clone(),
    })
}

fn to_brackets(entries: &[StampTaxBracketEntry]) -> Result<Vec<StampTaxBracket>, JLawError> {
    Ok(entries
        .iter()
        .map(|entry| StampTaxBracket {
            label: entry.label.clone(),
            amount_from: entry.amount_from,
            amount_to_inclusive: entry.amount_to_inclusive,
            tax_amount: entry.tax_amount,
        })
        .collect())
}

fn validate_brackets(
    document_code: StampTaxDocumentCode,
    brackets: &[StampTaxBracket],
) -> Result<(), JLawError> {
    let mut previous_to: Option<u64> = None;
    for bracket in brackets {
        if let Some(previous) = previous_to {
            if bracket.amount_from <= previous {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: document_code.to_string(),
                    reason: format!(
                        "ブラケットが重複または未整列です: {} starts at {} after {}",
                        bracket.label, bracket.amount_from, previous
                    ),
                }));
            }
            if bracket.amount_from != previous.saturating_add(1) {
                return Err(registry_parse_error(InputError::InvalidStampTaxInput {
                    field: document_code.to_string(),
                    reason: format!(
                        "ブラケットに空白があります: previous_to={}, next_from={}",
                        previous, bracket.amount_from
                    ),
                }));
            }
        }
        previous_to = bracket.amount_to_inclusive;
    }
    Ok(())
}

fn registry_parse_error(err: impl Into<JLawError>) -> JLawError {
    match err.into() {
        JLawError::Input(inner) => RegistryError::ParseError {
            path: PATH.into(),
            cause: inner.to_string(),
        }
        .into(),
        other => other,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::stamp_tax_schema::{StampTaxHistoryEntry, StampTaxParamsEntry, StampTaxRegistry};

    fn make_registry(entries: Vec<StampTaxHistoryEntry>) -> StampTaxRegistry {
        StampTaxRegistry {
            domain: "stamp_tax".into(),
            history: entries,
        }
    }

    fn make_entry(from: &str, until: Option<&str>) -> StampTaxHistoryEntry {
        StampTaxHistoryEntry {
            effective_from: from.into(),
            effective_until: until.map(|s| s.into()),
            params: StampTaxParamsEntry {
                documents: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn registry_validation_passes_for_current_data() {
        let json_str = include_str!("../data/stamp_tax/stamp_tax.json");
        let registry: StampTaxRegistry = serde_json::from_str(json_str).unwrap();
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_detects_overlap() {
        let reg = make_registry(vec![
            make_entry("2014-04-01", Some("2020-01-15")),
            make_entry("2020-01-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn registry_validation_detects_gap() {
        let reg = make_registry(vec![
            make_entry("2014-04-01", Some("2019-12-31")),
            make_entry("2020-01-03", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn registry_validation_detects_open_ended_before_next() {
        let reg = make_registry(vec![
            make_entry("2014-04-01", None),
            make_entry("2020-01-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn load_2024_params() {
        let params = load_stamp_tax_params(LegalDate::new(2024, 1, 1)).unwrap();
        assert!(params
            .documents
            .contains_key(&StampTaxDocumentCode::Article1RealEstateTransfer));
        assert!(params
            .documents
            .contains_key(&StampTaxDocumentCode::Article20SealBook));
    }

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_stamp_tax_params(LegalDate::new(2014, 3, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }
}
