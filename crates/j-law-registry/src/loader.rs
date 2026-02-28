use crate::schema::{BrokerageFeeRegistry, HistoryEntry};
use j_law_core::domains::real_estate::params::{
    BrokerageFeeParams, LowCostSpecialParams, TierParam,
};
use j_law_core::{InputError, JLawError, RegistryError};

/// `brokerage_fee.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
/// - Registry データの不整合 → `RegistryError` でパニック（起動時バリデーション）
pub fn load_brokerage_fee_params(
    target_date: (u16, u8, u8),
) -> Result<BrokerageFeeParams, JLawError> {
    let json_str = include_str!("../data/real_estate/brokerage_fee.json");

    let registry: BrokerageFeeRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::FileNotFound {
            path: format!("real_estate/brokerage_fee.json: {}", e),
        })?;

    let date_str = format!(
        "{:04}-{:02}-{:02}",
        target_date.0, target_date.1, target_date.2
    );

    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    Ok(to_params(entry))
}

/// `date_str` ("YYYY-MM-DD") に対応する履歴エントリを返す。
fn find_entry<'a>(registry: &'a BrokerageFeeRegistry, date_str: &str) -> Option<&'a HistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &HistoryEntry) -> BrokerageFeeParams {
    let tiers = entry
        .params
        .tiers
        .iter()
        .map(|t| TierParam {
            label: t.label.clone(),
            price_from: t.price_from,
            price_to_inclusive: t.price_to_inclusive,
            rate_numer: t.rate.numer,
            rate_denom: t.rate.denom,
        })
        .collect();

    let low_cost_special = entry
        .params
        .low_cost_special
        .as_ref()
        .map(|s| LowCostSpecialParams {
            price_ceiling_inclusive: s.price_ceiling_inclusive,
            fee_ceiling_exclusive_tax: s.fee_ceiling_exclusive_tax,
        });

    BrokerageFeeParams {
        tiers,
        tax_numer: entry.params.consumption_tax.numer,
        tax_denom: entry.params.consumption_tax.denom,
        low_cost_special,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // テストコードでは unwrap 使用を許可
mod tests {
    use super::*;
    use crate::validator::validate;

    #[test]
    fn load_2024_active_params() {
        let params = load_brokerage_fee_params((2024, 8, 1)).unwrap();
        assert_eq!(params.tiers.len(), 3);
        assert_eq!(params.tax_numer, 10);
        assert_eq!(params.tax_denom, 100);
        assert!(params.low_cost_special.is_some());
        let special = params.low_cost_special.unwrap();
        assert_eq!(special.price_ceiling_inclusive, 8_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 330_000);
    }

    #[test]
    fn load_2019_superseded_params() {
        let params = load_brokerage_fee_params((2019, 12, 1)).unwrap();
        assert_eq!(params.tiers.len(), 3);
        assert!(params.low_cost_special.is_none());
    }

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_brokerage_fee_params((2019, 9, 30));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn boundary_2024_07_01_uses_new_params() {
        let params = load_brokerage_fee_params((2024, 7, 1)).unwrap();
        assert!(params.low_cost_special.is_some());
    }

    #[test]
    fn boundary_2024_06_30_uses_old_params() {
        let params = load_brokerage_fee_params((2024, 6, 30)).unwrap();
        assert!(params.low_cost_special.is_none());
    }

    #[test]
    fn registry_validation_passes() {
        let json_str = include_str!("../data/real_estate/brokerage_fee.json");
        let registry: BrokerageFeeRegistry = serde_json::from_str(json_str).unwrap();
        validate(&registry).unwrap();
    }
}
