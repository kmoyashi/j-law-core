use crate::consumption_tax_schema::{ConsumptionTaxHistoryEntry, ConsumptionTaxRegistry};
use j_law_core::domains::consumption_tax::params::{ConsumptionTaxParams, ConsumptionTaxRate};
use j_law_core::types::date::LegalDate;
use j_law_core::{JLawError, RegistryError};

const PATH: &str = "consumption_tax/consumption_tax.json";

/// `consumption_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 消費税法 第29条
///
/// # 日付の範囲外について
/// 消費税導入前（1989年4月1日以前）の日付には消費税が存在しないため、
/// エラーではなく税率0%のパラメータを返す。
/// 返却される `standard_rate` は `{ numer: 0, denom: 1 }`（0/1 = 0%）。
/// `denom: 1` はゼロ除算が起きない最小の分母であり、「導入前＝非課税」を意味する。
pub fn load_consumption_tax_params(
    target_date: LegalDate,
) -> Result<ConsumptionTaxParams, JLawError> {
    target_date.validate()?;

    let json_str = include_str!("../data/consumption_tax/consumption_tax.json");

    let registry: ConsumptionTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: PATH.into(),
            cause: e.to_string(),
        })?;
    validate_registry(&registry)?;

    let date_str = target_date.to_date_str();

    match find_entry(&registry, &date_str) {
        Some(entry) => Ok(to_params(entry)),
        // 消費税導入前（1989-04-01以前）: エラーではなく0%を返す
        None => Ok(ConsumptionTaxParams {
            standard_rate: ConsumptionTaxRate { numer: 0, denom: 1 },
            reduced_rate: None,
        }),
    }
}

/// `date_str` ("YYYY-MM-DD") に対応する履歴エントリを返す。
fn find_entry<'a>(
    registry: &'a ConsumptionTaxRegistry,
    date_str: &str,
) -> Option<&'a ConsumptionTaxHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

/// `ConsumptionTaxRegistry` の整合性を検証する。
///
/// # 検証内容
/// - 適用期間の重複（Overlap）
/// - 適用期間の空白（Gap）
/// - 分母ゼロ（standard_rate.denom / reduced_rate.denom）
fn validate_registry(registry: &ConsumptionTaxRegistry) -> Result<(), RegistryError> {
    let domain = &registry.domain;

    // 分母ゼロチェック
    for (i, entry) in registry.history.iter().enumerate() {
        if entry.params.standard_rate.denom == 0 {
            return Err(RegistryError::ZeroDenominator {
                path: format!("{domain}/history[{i}]/standard_rate.denom"),
            });
        }
        if let Some(reduced) = &entry.params.reduced_rate {
            if reduced.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!("{domain}/history[{i}]/reduced_rate.denom"),
                });
            }
        }
    }

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

fn to_params(entry: &ConsumptionTaxHistoryEntry) -> ConsumptionTaxParams {
    ConsumptionTaxParams {
        standard_rate: ConsumptionTaxRate {
            numer: entry.params.standard_rate.numer,
            denom: entry.params.standard_rate.denom,
        },
        reduced_rate: entry
            .params
            .reduced_rate
            .as_ref()
            .map(|r| ConsumptionTaxRate {
                numer: r.numer,
                denom: r.denom,
            }),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // テストコードでは unwrap 使用を許可
mod tests {
    use super::*;
    use crate::consumption_tax_schema::{
        ConsumptionTaxCitationEntry, ConsumptionTaxFraction, ConsumptionTaxHistoryEntry,
        ConsumptionTaxParamsEntry, ConsumptionTaxRegistry,
    };

    fn make_registry(entries: Vec<ConsumptionTaxHistoryEntry>) -> ConsumptionTaxRegistry {
        ConsumptionTaxRegistry {
            domain: "consumption_tax".into(),
            history: entries,
        }
    }

    fn make_entry(from: &str, until: Option<&str>) -> ConsumptionTaxHistoryEntry {
        ConsumptionTaxHistoryEntry {
            effective_from: from.into(),
            effective_until: until.map(|s| s.into()),
            status: "active".into(),
            citation: ConsumptionTaxCitationEntry {
                law_id: "test".into(),
                law_name: "消費税法".into(),
                article: 29,
                paragraph: None,
                ministry: "財務省".into(),
            },
            params: ConsumptionTaxParamsEntry {
                standard_rate: ConsumptionTaxFraction {
                    numer: 10,
                    denom: 100,
                },
                reduced_rate: None,
            },
        }
    }

    #[test]
    fn registry_validation_passes_for_current_data() {
        let json_str = include_str!("../data/consumption_tax/consumption_tax.json");
        let registry: ConsumptionTaxRegistry = serde_json::from_str(json_str).unwrap();
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_detects_overlap() {
        let reg = make_registry(vec![
            make_entry("1989-04-01", Some("1997-04-15")),
            make_entry("1997-04-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn registry_validation_detects_gap() {
        let reg = make_registry(vec![
            make_entry("1989-04-01", Some("1997-03-31")),
            make_entry("1997-04-03", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn registry_validation_detects_zero_denominator_standard_rate() {
        let mut reg = make_registry(vec![make_entry("1989-04-01", None)]);
        reg.history[0].params.standard_rate.denom = 0;
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::ZeroDenominator { .. }));
    }

    #[test]
    fn registry_validation_detects_zero_denominator_reduced_rate() {
        let mut reg = make_registry(vec![make_entry("2019-10-01", None)]);
        reg.history[0].params.reduced_rate = Some(ConsumptionTaxFraction { numer: 8, denom: 0 });
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::ZeroDenominator { .. }));
    }

    #[test]
    fn registry_validation_detects_open_ended_before_next() {
        let reg = make_registry(vec![
            make_entry("1989-04-01", None), // effective_until なし（open-ended）
            make_entry("1997-04-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn load_3pct_1990() {
        let params = load_consumption_tax_params(LegalDate::new(1990, 1, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 3);
        assert_eq!(params.standard_rate.denom, 100);
        assert!(params.reduced_rate.is_none());
    }

    #[test]
    fn load_5pct_2000() {
        let params = load_consumption_tax_params(LegalDate::new(2000, 1, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 5);
        assert_eq!(params.standard_rate.denom, 100);
        assert!(params.reduced_rate.is_none());
    }

    #[test]
    fn load_8pct_2016() {
        let params = load_consumption_tax_params(LegalDate::new(2016, 1, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 8);
        assert_eq!(params.standard_rate.denom, 100);
        assert!(params.reduced_rate.is_none());
    }

    #[test]
    fn load_10pct_with_reduced_2020() {
        let params = load_consumption_tax_params(LegalDate::new(2020, 1, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 10);
        assert_eq!(params.standard_rate.denom, 100);
        let reduced = params.reduced_rate.unwrap();
        assert_eq!(reduced.numer, 8);
        assert_eq!(reduced.denom, 100);
    }

    #[test]
    fn before_introduction_returns_zero_rate() {
        // 消費税導入前はエラーではなく0%を返す
        let params = load_consumption_tax_params(LegalDate::new(1988, 12, 31)).unwrap();
        assert_eq!(params.standard_rate.numer, 0);
        assert!(params.reduced_rate.is_none());
    }

    #[test]
    fn boundary_1989_04_01_is_3pct() {
        let params = load_consumption_tax_params(LegalDate::new(1989, 4, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 3);
    }

    #[test]
    fn boundary_1997_03_31_is_3pct() {
        // 1997-03-31 まで3%
        let params = load_consumption_tax_params(LegalDate::new(1997, 3, 31)).unwrap();
        assert_eq!(params.standard_rate.numer, 3);
    }

    #[test]
    fn boundary_1997_04_01_is_5pct() {
        // 1997-04-01 から5%
        let params = load_consumption_tax_params(LegalDate::new(1997, 4, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 5);
    }

    #[test]
    fn boundary_2014_04_01_is_8pct() {
        let params = load_consumption_tax_params(LegalDate::new(2014, 4, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 8);
    }

    #[test]
    fn boundary_2019_09_30_is_8pct() {
        let params = load_consumption_tax_params(LegalDate::new(2019, 9, 30)).unwrap();
        assert_eq!(params.standard_rate.numer, 8);
        assert!(params.reduced_rate.is_none());
    }

    #[test]
    fn boundary_2019_10_01_is_10pct() {
        let params = load_consumption_tax_params(LegalDate::new(2019, 10, 1)).unwrap();
        assert_eq!(params.standard_rate.numer, 10);
        assert!(params.reduced_rate.is_some());
    }

    #[test]
    fn invalid_date_returns_input_error() {
        let result = load_consumption_tax_params(LegalDate::new(2024, 13, 1));
        assert!(matches!(
            result,
            Err(JLawError::Input(j_law_core::InputError::InvalidDate { .. }))
        ));
    }
}
