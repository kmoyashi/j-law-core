use crate::income_tax_schema::{IncomeTaxHistoryEntry, IncomeTaxRegistry};
use j_law_core::domains::income_tax::params::{
    IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

const PATH: &str = "income_tax/income_tax.json";

/// `income_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 所得税法 第89条第1項
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
pub fn load_income_tax_params(target_date: LegalDate) -> Result<IncomeTaxParams, JLawError> {
    target_date.validate()?;

    let json_str = include_str!("../data/income_tax/income_tax.json");

    let registry: IncomeTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: PATH.into(),
            cause: e.to_string(),
        })?;
    validate_registry(&registry)?;

    let date_str = target_date.to_date_str();

    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    Ok(to_params(entry))
}

/// `IncomeTaxRegistry` の整合性を検証する。
///
/// # 検証内容
/// - 適用期間の重複（Overlap）
/// - 適用期間の空白（Gap）
/// - 分母ゼロ（ブラケット rate.denom / reconstruction_tax rate.denom）
fn validate_registry(registry: &IncomeTaxRegistry) -> Result<(), RegistryError> {
    let domain = &registry.domain;

    // 分母ゼロチェック
    for (i, entry) in registry.history.iter().enumerate() {
        for (j, bracket) in entry.params.brackets.iter().enumerate() {
            if bracket.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!("{domain}/history[{i}]/brackets[{j}]/rate.denom"),
                });
            }
        }
        if let Some(rt) = &entry.params.reconstruction_tax {
            if rt.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!("{domain}/history[{i}]/reconstruction_tax/rate.denom"),
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

fn find_entry<'a>(
    registry: &'a IncomeTaxRegistry,
    date_str: &str,
) -> Option<&'a IncomeTaxHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &IncomeTaxHistoryEntry) -> IncomeTaxParams {
    let brackets = entry
        .params
        .brackets
        .iter()
        .map(|b| IncomeTaxBracket {
            label: b.label.clone(),
            income_from: b.income_from,
            income_to_inclusive: b.income_to_inclusive,
            rate_numer: b.rate.numer,
            rate_denom: b.rate.denom,
            deduction: b.deduction,
        })
        .collect();

    let reconstruction_tax =
        entry
            .params
            .reconstruction_tax
            .as_ref()
            .map(|rt| ReconstructionTaxParams {
                rate_numer: rt.rate.numer,
                rate_denom: rt.rate.denom,
                effective_from_year: rt.effective_from_year,
                effective_to_year_inclusive: rt.effective_to_year_inclusive,
            });

    IncomeTaxParams {
        brackets,
        reconstruction_tax,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // テストコードでは unwrap 使用を許可
mod tests {
    use super::*;
    use crate::income_tax_schema::{
        IncomeTaxBracketEntry, IncomeTaxHistoryEntry, IncomeTaxParamsEntry, IncomeTaxRegistry,
        ReconstructionTaxEntry,
    };
    use crate::schema::Fraction;

    fn make_registry(entries: Vec<IncomeTaxHistoryEntry>) -> IncomeTaxRegistry {
        IncomeTaxRegistry {
            domain: "income_tax".into(),
            history: entries,
        }
    }

    fn make_entry(from: &str, until: Option<&str>) -> IncomeTaxHistoryEntry {
        IncomeTaxHistoryEntry {
            effective_from: from.into(),
            effective_until: until.map(|s| s.into()),
            params: IncomeTaxParamsEntry {
                brackets: vec![IncomeTaxBracketEntry {
                    label: "一律".into(),
                    income_from: 0,
                    income_to_inclusive: None,
                    rate: Fraction { numer: 10, denom: 100 },
                    deduction: 0,
                }],
                reconstruction_tax: None,
            },
        }
    }

    #[test]
    fn registry_validation_passes_for_current_data() {
        let json_str = include_str!("../data/income_tax/income_tax.json");
        let registry: IncomeTaxRegistry = serde_json::from_str(json_str).unwrap();
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_detects_overlap() {
        let reg = make_registry(vec![
            make_entry("1989-01-01", Some("1995-01-15")),
            make_entry("1995-01-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn registry_validation_detects_gap() {
        let reg = make_registry(vec![
            make_entry("1989-01-01", Some("1994-12-31")),
            make_entry("1995-01-03", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn registry_validation_detects_open_ended_before_next() {
        let reg = make_registry(vec![
            make_entry("1989-01-01", None),
            make_entry("1995-01-01", None),
        ]);
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn registry_validation_detects_zero_denominator_bracket() {
        let mut reg = make_registry(vec![make_entry("1989-01-01", None)]);
        reg.history[0].params.brackets[0].rate.denom = 0;
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::ZeroDenominator { .. }));
    }

    #[test]
    fn registry_validation_detects_zero_denominator_reconstruction_tax() {
        let mut reg = make_registry(vec![make_entry("2013-01-01", None)]);
        reg.history[0].params.reconstruction_tax = Some(ReconstructionTaxEntry {
            rate: Fraction { numer: 21, denom: 0 },
            effective_from_year: 2013,
            effective_to_year_inclusive: 2037,
        });
        let err = validate_registry(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::ZeroDenominator { .. }));
    }

    // ─── 現行（2015年〜）7段階 ────────────────────────────────────────────────

    #[test]
    fn load_2024_params() {
        let params = load_income_tax_params(LegalDate::new(2024, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 7);
        assert!(params.reconstruction_tax.is_some());
        let rt = params.reconstruction_tax.unwrap();
        assert_eq!(rt.rate_numer, 21);
        assert_eq!(rt.rate_denom, 1000);
    }

    #[test]
    fn load_2015_params() {
        let params = load_income_tax_params(LegalDate::new(2015, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 7);
    }

    // ─── 平成19〜26年分（2007〜2014年）6段階 ──────────────────────────────────

    #[test]
    fn load_2007_params() {
        let params = load_income_tax_params(LegalDate::new(2010, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 6);
        // 最高税率は40%
        let top = params.brackets.last().unwrap();
        assert_eq!(top.rate_numer, 40);
        assert_eq!(top.deduction, 2_796_000);
    }

    #[test]
    fn load_2007_params_no_reconstruction_tax() {
        // 復興特別所得税は2013年開始。2007〜2012年は None であること
        let params = load_income_tax_params(LegalDate::new(2010, 1, 1)).unwrap();
        assert!(params.reconstruction_tax.is_none());
    }

    #[test]
    fn boundary_2012_12_31_no_reconstruction_tax() {
        let params = load_income_tax_params(LegalDate::new(2012, 12, 31)).unwrap();
        assert_eq!(params.brackets.len(), 6);
        assert!(params.reconstruction_tax.is_none());
    }

    #[test]
    fn boundary_2013_01_01_has_reconstruction_tax() {
        let params = load_income_tax_params(LegalDate::new(2013, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 6);
        assert!(params.reconstruction_tax.is_some());
    }

    #[test]
    fn boundary_2014_12_31() {
        let params = load_income_tax_params(LegalDate::new(2014, 12, 31)).unwrap();
        assert_eq!(params.brackets.len(), 6);
        assert!(params.reconstruction_tax.is_some());
    }

    #[test]
    fn boundary_2015_01_01() {
        let params = load_income_tax_params(LegalDate::new(2015, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 7);
    }

    // ─── 平成11〜18年分（1999〜2006年）4段階 ──────────────────────────────────

    #[test]
    fn load_1999_params() {
        let params = load_income_tax_params(LegalDate::new(2003, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 4);
        // 最高税率は37%
        let top = params.brackets.last().unwrap();
        assert_eq!(top.rate_numer, 37);
        assert_eq!(top.deduction, 2_490_000);
        assert!(params.reconstruction_tax.is_none());
    }

    #[test]
    fn boundary_2006_12_31() {
        let params = load_income_tax_params(LegalDate::new(2006, 12, 31)).unwrap();
        assert_eq!(params.brackets.len(), 4);
    }

    #[test]
    fn boundary_2007_01_01() {
        let params = load_income_tax_params(LegalDate::new(2007, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 6);
    }

    // ─── 平成7〜10年分（1995〜1998年）5段階 ───────────────────────────────────

    #[test]
    fn load_1995_params() {
        let params = load_income_tax_params(LegalDate::new(1997, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 5);
        // 最高税率は50%
        let top = params.brackets.last().unwrap();
        assert_eq!(top.rate_numer, 50);
        assert_eq!(top.deduction, 6_030_000);
        assert!(params.reconstruction_tax.is_none());
    }

    #[test]
    fn boundary_1998_12_31() {
        let params = load_income_tax_params(LegalDate::new(1998, 12, 31)).unwrap();
        assert_eq!(params.brackets.len(), 5);
    }

    #[test]
    fn boundary_1999_01_01() {
        let params = load_income_tax_params(LegalDate::new(1999, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 4);
    }

    // 1995改正では10%上限が330万円（1989改正の300万円から変更）
    #[test]
    fn boundary_1995_01_01_bracket_change() {
        let params = load_income_tax_params(LegalDate::new(1995, 1, 1)).unwrap();
        let first = &params.brackets[0];
        assert_eq!(first.income_to_inclusive, Some(3_300_000));
        assert_eq!(first.rate_numer, 10);
        assert_eq!(first.deduction, 0);
    }

    // ─── 平成元〜6年分（1989〜1994年）5段階 ───────────────────────────────────

    #[test]
    fn load_1989_params() {
        let params = load_income_tax_params(LegalDate::new(1990, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 5);
        // 最高税率は50%、1989期間は10%上限が300万円
        let first = &params.brackets[0];
        assert_eq!(first.income_to_inclusive, Some(3_000_000));
        assert_eq!(first.rate_numer, 10);
        assert!(params.reconstruction_tax.is_none());
    }

    #[test]
    fn invalid_date_returns_input_error() {
        let result = load_income_tax_params(LegalDate::new(2024, 2, 30));
        assert!(matches!(
            result,
            Err(JLawError::Input(j_law_core::InputError::InvalidDate { .. }))
        ));
    }

    #[test]
    fn boundary_1989_01_01() {
        let params = load_income_tax_params(LegalDate::new(1989, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 5);
    }

    #[test]
    fn boundary_1994_12_31() {
        let params = load_income_tax_params(LegalDate::new(1994, 12, 31)).unwrap();
        // 1989ブラケット: 10%上限は300万円
        let first = &params.brackets[0];
        assert_eq!(first.income_to_inclusive, Some(3_000_000));
    }

    // ─── 範囲外 ──────────────────────────────────────────────────────────────

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_income_tax_params(LegalDate::new(1988, 12, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn registry_data_integrity_check() {
        let json_str = include_str!("../data/income_tax/income_tax.json");
        let registry: IncomeTaxRegistry = serde_json::from_str(json_str).unwrap();

        // 基本的な整合性チェック
        assert!(!registry.history.is_empty(), "Registry should not be empty");

        for entry in &registry.history {
            // 分母ゼロチェック
            for bracket in &entry.params.brackets {
                assert_ne!(bracket.rate.denom, 0, "Rate denominator must not be zero");
            }

            if let Some(rt) = &entry.params.reconstruction_tax {
                assert_ne!(
                    rt.rate.denom, 0,
                    "Reconstruction tax denominator must not be zero"
                );
            }
        }
    }
}
