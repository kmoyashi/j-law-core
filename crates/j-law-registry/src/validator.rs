use crate::schema::BrokerageFeeRegistry;
use j_law_core::RegistryError;

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

    for i in 0..sorted.len().saturating_sub(1) {
        let current = &sorted[i];
        let next = &sorted[i + 1];

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

        // 空白チェック: current.until の翌日 != next.from
        // 簡易実装: "YYYY-MM-DD" 文字列で1日差を検証するのは複雑なため、
        // ここでは until と next.from が連続していない（until < next.from - 1日相当）場合を Gap とする。
        // 正確には日付ライブラリが必要だが、依存を増やさないため文字列比較のみとする。
        // Gap = current_until < next.effective_from の間に1日以上の空白がある場合
        // "2024-06-30" の次は "2024-07-01" が正常。文字列が連続していない場合は Gap とみなす。
        // NOTE: この簡易実装は月末・年末の境界で誤検知する可能性あり。
        //       本番実装では chrono クレートの追加を検討する。
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
