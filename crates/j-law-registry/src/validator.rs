use crate::schema::BrokerageFeeRegistry;
use j_law_core::{LegalDate, RegistryError};

/// Registry データの整合性を検証する。
///
/// # 検証内容
/// - 適用期間の重複（Overlap）
/// - 適用期間の空白（Gap）
/// - 分母ゼロ
///
/// # エラー
/// 不正なデータを検出した場合は起動時にパニックさせてよい（設定ファイルエラー）。
#[allow(dead_code)] // テストでのみ使用されているが、将来的に他クレートから使用される可能性あり
pub fn validate(registry: &BrokerageFeeRegistry) -> Result<(), RegistryError> {
    let domain = &registry.domain;

    // 各 tier の denom ゼロチェック
    for entry in &registry.history {
        for tier in &entry.params.tiers {
            if tier.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!("{}/{}/rate.denom", domain, tier.label),
                });
            }
        }
    }

    // 期間の重複・空白チェック（日付文字列を辞書順比較）
    // history は effective_from 昇順であることを前提とする
    let mut sorted = registry.history.clone();
    sorted.sort_by(|a, b| a.effective_from.cmp(&b.effective_from));

    for [current, next] in sorted.array_windows::<2>() {
        let current_until = match &current.effective_until {
            Some(d) => d.clone(),
            // active（現行）エントリは期間が終わっていないので次エントリと重複チェック不要
            None => continue,
        };

        // 重複チェック: current.until >= next.from
        if current_until >= next.effective_from {
            return Err(RegistryError::PeriodOverlap {
                domain: domain.clone(),
                from: next.effective_from.clone(),
                until: current_until.clone(),
            });
        }

        // 空白チェック: current.until の翌日 == next.from であることを検証する。
        // LegalDate::next_day() による純粋算術ベースの日付演算を使用する。
        let until_date = match LegalDate::from_date_str(&current_until) {
            Some(d) => d,
            None => {
                return Err(RegistryError::InvalidDateFormat {
                    domain: domain.clone(),
                    value: current_until.clone(),
                });
            }
        };
        let expected_next_from = until_date.next_day().to_date_str();
        if expected_next_from != next.effective_from {
            return Err(RegistryError::PeriodGap {
                domain: domain.clone(),
                end: current_until.clone(),
                next_start: next.effective_from.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // テストコードでは unwrap 使用を許可
mod tests {
    use super::*;
    use crate::schema::{BrokerageFeeRegistry, CitationEntry, HistoryEntry, ParamsEntry};

    fn make_registry(entries: Vec<HistoryEntry>) -> BrokerageFeeRegistry {
        BrokerageFeeRegistry {
            domain: "test".into(),
            history: entries,
        }
    }

    fn make_entry(from: &str, until: Option<&str>) -> HistoryEntry {
        HistoryEntry {
            effective_from: from.into(),
            effective_until: until.map(|s: &str| s.into()),
            status: "active".into(),
            citation: CitationEntry {
                law_id: "test".into(),
                law_name: "test law".into(),
                article: 1,
                paragraph: None,
                ministry: "test".into(),
            },
            params: ParamsEntry {
                tiers: vec![],
                low_cost_special: None,
            },
        }
    }

    #[test]
    fn valid_registry_passes() {
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-06-30")),
            make_entry("2024-07-01", None),
        ]);
        assert!(validate(&reg).is_ok());
    }

    #[test]
    fn overlap_detected() {
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-07-15")),
            make_entry("2024-07-01", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodOverlap { .. }));
    }

    #[test]
    fn gap_detected_simple() {
        // 2024-06-30 の翌日は 2024-07-01 だが、次エントリが 2024-07-02 → Gap
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-06-30")),
            make_entry("2024-07-02", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn gap_detected_month_boundary() {
        // 月末境界: 2024-06-30 の翌日は 2024-07-01。2024-08-01 では Gap
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-06-30")),
            make_entry("2024-08-01", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn gap_detected_year_boundary() {
        // 年末境界: 2023-12-31 の翌日は 2024-01-01。2024-01-02 では Gap
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2023-12-31")),
            make_entry("2024-01-02", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::PeriodGap { .. }));
    }

    #[test]
    fn valid_year_boundary_no_gap() {
        // 年末境界: 2023-12-31 の翌日が 2024-01-01 → ギャップなし
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2023-12-31")),
            make_entry("2024-01-01", None),
        ]);
        assert!(validate(&reg).is_ok());
    }

    #[test]
    fn malformed_date_rejected() {
        // "2024-00-30" は辞書順で "2024-07-01" より小さいため重複チェックを通過し、
        // from_date_str で month=0 が弾かれて InvalidDateFormat になる
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-00-30")),
            make_entry("2024-07-01", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::InvalidDateFormat { .. }));
    }

    #[test]
    fn impossible_date_rejected() {
        // 2月30日は存在しない — from_date_str の厳密バリデーションで拒否される
        let reg: BrokerageFeeRegistry = make_registry(vec![
            make_entry("2019-10-01", Some("2024-02-30")),
            make_entry("2024-07-01", None),
        ]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::InvalidDateFormat { .. }));
    }

    #[test]
    fn zero_denom_detected() {
        use crate::schema::{Fraction, TierParam};
        let mut entry: HistoryEntry = make_entry("2024-07-01", None);
        entry.params.tiers.push(TierParam {
            label: "tier1".into(),
            price_from: 0,
            price_to_inclusive: Some(2_000_000),
            rate: Fraction { numer: 5, denom: 0 }, // denom ゼロで異常
        });
        let reg: BrokerageFeeRegistry = make_registry(vec![entry]);
        let err: RegistryError = validate(&reg).unwrap_err();
        assert!(matches!(err, RegistryError::ZeroDenominator { .. }));
    }
}
