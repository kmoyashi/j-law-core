use crate::income_tax_schema::{IncomeTaxHistoryEntry, IncomeTaxRegistry};
use j_law_core::domains::income_tax::params::{
    IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `income_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 所得税法 第89条第1項
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
pub fn load_income_tax_params(target_date: LegalDate) -> Result<IncomeTaxParams, JLawError> {
    let json_str = include_str!("../data/income_tax/income_tax.json");

    let registry: IncomeTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "income_tax/income_tax.json".into(),
            cause: e.to_string(),
        })?;

    let date_str = target_date.to_date_str();

    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    Ok(to_params(entry))
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

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_income_tax_params(LegalDate::new(2014, 12, 31));
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
