use crate::stamp_tax_schema::{StampTaxHistoryEntry, StampTaxRegistry};
use j_law_core::domains::stamp_tax::params::{StampTaxBracket, StampTaxParams};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `stamp_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
pub fn load_stamp_tax_params(target_date: LegalDate) -> Result<StampTaxParams, JLawError> {
    let json_str = include_str!("../data/stamp_tax/stamp_tax.json");

    let registry: StampTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "stamp_tax/stamp_tax.json".into(),
            cause: e.to_string(),
        })?;

    let date_str = target_date.to_date_str();

    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    Ok(to_params(entry))
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

fn to_params(entry: &StampTaxHistoryEntry) -> StampTaxParams {
    let brackets = entry
        .params
        .brackets
        .iter()
        .map(|b| StampTaxBracket {
            label: b.label.clone(),
            amount_from: b.amount_from,
            amount_to_inclusive: b.amount_to_inclusive,
            tax_amount: b.tax_amount,
            reduced_tax_amount: b.reduced_tax_amount,
        })
        .collect();

    StampTaxParams {
        brackets,
        reduced_rate_from: entry.params.reduced_rate_from.clone(),
        reduced_rate_until: entry.params.reduced_rate_until.clone(),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // テストコードでは unwrap 使用を許可
mod tests {
    use super::*;

    #[test]
    fn load_2024_params() {
        let params = load_stamp_tax_params(LegalDate::new(2024, 1, 1)).unwrap();
        assert_eq!(params.brackets.len(), 12);
        assert!(params.reduced_rate_from.is_some());
        assert!(params.reduced_rate_until.is_some());
    }

    #[test]
    fn load_2014_params() {
        let params = load_stamp_tax_params(LegalDate::new(2014, 4, 1)).unwrap();
        assert_eq!(params.brackets.len(), 12);
    }

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_stamp_tax_params(LegalDate::new(2014, 3, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn registry_data_integrity_check() {
        let json_str = include_str!("../data/stamp_tax/stamp_tax.json");
        let registry: StampTaxRegistry = serde_json::from_str(json_str).unwrap();

        // 基本的な整合性チェック
        assert!(!registry.history.is_empty(), "Registry should not be empty");

        for entry in &registry.history {
            // ブラケットが存在することを確認
            assert!(
                !entry.params.brackets.is_empty(),
                "Brackets should not be empty"
            );
        }
    }
}
