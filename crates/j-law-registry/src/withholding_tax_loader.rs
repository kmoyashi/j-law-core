use std::str::FromStr;

use crate::withholding_tax_schema::{WithholdingTaxHistoryEntry, WithholdingTaxRegistry};
use j_law_core::domains::withholding_tax::params::{
    WithholdingTaxCategoryParams, WithholdingTaxMethod, WithholdingTaxParams,
};
use j_law_core::domains::withholding_tax::WithholdingTaxCategory;
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `withholding_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 所得税法 第204条第1項
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
pub fn load_withholding_tax_params(
    target_date: LegalDate,
) -> Result<WithholdingTaxParams, JLawError> {
    target_date.validate()?;

    let json_str = include_str!("../data/withholding_tax/withholding_tax.json");

    let registry: WithholdingTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "withholding_tax/withholding_tax.json".into(),
            cause: e.to_string(),
        })?;
    validate_periods(&registry)?;

    let date_str = target_date.to_date_str();
    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    to_params(entry)
}

/// `WithholdingTaxRegistry` の適用期間の整合性を検証する。
///
/// # 検証内容
/// - 適用期間の重複（Overlap）
/// - 適用期間の空白（Gap）
///
/// 分母ゼロは `to_params()` で個別にチェックする。
fn validate_periods(registry: &WithholdingTaxRegistry) -> Result<(), RegistryError> {
    let domain = &registry.domain;

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
    registry: &'a WithholdingTaxRegistry,
    date_str: &str,
) -> Option<&'a WithholdingTaxHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &WithholdingTaxHistoryEntry) -> Result<WithholdingTaxParams, JLawError> {
    let categories = entry
        .params
        .categories
        .iter()
        .map(|category| {
            if category.method.base_rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!(
                        "withholding_tax/withholding_tax.json/{}/method/base_rate/denom",
                        category.code
                    ),
                }
                .into());
            }
            if category.method.excess_rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!(
                        "withholding_tax/withholding_tax.json/{}/method/excess_rate/denom",
                        category.code
                    ),
                }
                .into());
            }
            if category.method.kind != "two_tier" {
                return Err(RegistryError::ParseError {
                    path: format!("withholding_tax/withholding_tax.json/{}", category.code),
                    cause: format!("未知の計算方式です: {}", category.method.kind),
                }
                .into());
            }

            Ok(WithholdingTaxCategoryParams {
                category: parse_category_code(&category.code)?,
                label: category.label.clone(),
                method: WithholdingTaxMethod::TwoTier {
                    threshold: category.method.threshold,
                    base_rate_numer: category.method.base_rate.numer,
                    base_rate_denom: category.method.base_rate.denom,
                    excess_rate_numer: category.method.excess_rate.numer,
                    excess_rate_denom: category.method.excess_rate.denom,
                },
                submission_prize_exemption_threshold: category.submission_prize_exemption_threshold,
            })
        })
        .collect::<Result<Vec<_>, JLawError>>()?;

    Ok(WithholdingTaxParams { categories })
}

fn parse_category_code(code: &str) -> Result<WithholdingTaxCategory, JLawError> {
    WithholdingTaxCategory::from_str(code).map_err(|err| {
        RegistryError::ParseError {
            path: format!("withholding_tax/withholding_tax.json/{code}"),
            cause: err.to_string(),
        }
        .into()
    })
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::withholding_tax_schema::{
        WithholdingTaxCategoryEntry, WithholdingTaxCitationEntry, WithholdingTaxFraction,
        WithholdingTaxHistoryEntry, WithholdingTaxParamsEntry, WithholdingTaxRegistry,
        WithholdingTaxTwoTierMethodEntry,
    };

    fn make_registry(entries: Vec<WithholdingTaxHistoryEntry>) -> WithholdingTaxRegistry {
        WithholdingTaxRegistry {
            domain: "withholding_tax".into(),
            history: entries,
        }
    }

    fn make_entry(from: &str, until: Option<&str>) -> WithholdingTaxHistoryEntry {
        WithholdingTaxHistoryEntry {
            effective_from: from.into(),
            effective_until: until.map(|s| s.into()),
            status: "active".into(),
            citation: WithholdingTaxCitationEntry {
                law_name: "所得税法".into(),
                article: 204,
                paragraph: Some(1),
            },
            params: WithholdingTaxParamsEntry { categories: vec![] },
        }
    }

    #[test]
    fn registry_validation_passes_for_current_data() {
        let json_str = include_str!("../data/withholding_tax/withholding_tax.json");
        let registry: WithholdingTaxRegistry = serde_json::from_str(json_str).unwrap();
        assert!(validate_periods(&registry).is_ok());
    }

    #[test]
    fn registry_validation_detects_overlap() {
        let reg = make_registry(vec![
            make_entry("2013-01-01", Some("2030-01-15")),
            make_entry("2030-01-01", None),
        ]);
        let err = validate_periods(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn registry_validation_detects_gap() {
        let reg = make_registry(vec![
            make_entry("2013-01-01", Some("2029-12-31")),
            make_entry("2030-01-03", None),
        ]);
        let err = validate_periods(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn registry_validation_detects_open_ended_before_next() {
        let reg = make_registry(vec![
            make_entry("2013-01-01", None),
            make_entry("2030-01-01", None),
        ]);
        let err = validate_periods(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn load_2026_params() {
        let params = load_withholding_tax_params(LegalDate::new(2026, 1, 1)).unwrap();
        assert_eq!(params.categories.len(), 3);
    }

    #[test]
    fn manuscript_category_has_exemption_threshold() {
        let params = load_withholding_tax_params(LegalDate::new(2026, 1, 1)).unwrap();
        let manuscript = params
            .categories
            .iter()
            .find(|category| category.category == WithholdingTaxCategory::ManuscriptAndLecture)
            .unwrap();
        assert_eq!(
            manuscript.submission_prize_exemption_threshold,
            Some(50_000)
        );
    }

    #[test]
    fn date_before_period_is_out_of_range() {
        let result = load_withholding_tax_params(LegalDate::new(2012, 12, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn date_after_period_is_out_of_range() {
        let result = load_withholding_tax_params(LegalDate::new(2038, 1, 1));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn invalid_category_code_is_registry_parse_error() {
        let entry = WithholdingTaxHistoryEntry {
            effective_from: "2013-01-01".into(),
            effective_until: Some("2037-12-31".into()),
            status: "active".into(),
            citation: WithholdingTaxCitationEntry {
                law_name: "所得税法".into(),
                article: 204,
                paragraph: Some(1),
            },
            params: WithholdingTaxParamsEntry {
                categories: vec![WithholdingTaxCategoryEntry {
                    code: "unknown_category".into(),
                    label: "未知カテゴリ".into(),
                    method: WithholdingTaxTwoTierMethodEntry {
                        kind: "two_tier".into(),
                        threshold: 1_000_000,
                        base_rate: WithholdingTaxFraction {
                            numer: 1021,
                            denom: 10_000,
                        },
                        excess_rate: WithholdingTaxFraction {
                            numer: 2042,
                            denom: 10_000,
                        },
                    },
                    submission_prize_exemption_threshold: None,
                }],
            },
        };

        let result = to_params(&entry);
        assert!(matches!(
            result,
            Err(JLawError::Registry(RegistryError::ParseError { .. }))
        ));
    }
}
