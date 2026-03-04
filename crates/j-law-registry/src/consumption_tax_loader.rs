use crate::consumption_tax_schema::{ConsumptionTaxHistoryEntry, ConsumptionTaxRegistry};
use j_law_core::domains::consumption_tax::params::{ConsumptionTaxParams, ConsumptionTaxRate};
use j_law_core::types::date::LegalDate;
use j_law_core::{JLawError, RegistryError};

/// `consumption_tax.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 消費税法 第29条
///
/// # 日付の範囲外について
/// 消費税導入前（1989年4月1日以前）の日付には消費税が存在しないため、
/// エラーではなく税率0%のパラメータを返す。
pub fn load_consumption_tax_params(
    target_date: LegalDate,
) -> Result<ConsumptionTaxParams, JLawError> {
    let json_str = include_str!("../data/consumption_tax/consumption_tax.json");

    let registry: ConsumptionTaxRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::FileNotFound {
            path: format!("consumption_tax/consumption_tax.json: {}", e),
        })?;

    let date_str = target_date.to_date_str();

    match find_entry(&registry, &date_str) {
        Some(entry) => Ok(to_params(entry)),
        // 消費税導入前（1989-04-01以前）: エラーではなく0%を返す
        None => Ok(ConsumptionTaxParams {
            standard_rate: ConsumptionTaxRate { numer: 0, denom: 100 },
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
    fn boundary_1997_03_31_is_5pct() {
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
}
