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
