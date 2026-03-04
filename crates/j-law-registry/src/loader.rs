use crate::schema::{BrokerageFeeRegistry, HistoryEntry};
use j_law_core::domains::consumption_tax::params::{ConsumptionTaxParams, ConsumptionTaxRate};
use j_law_core::domains::real_estate::params::{
    BrokerageFeeParams, LowCostSpecialParams, TierParam,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `brokerage_fee.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
///
/// # エラー
/// - `target_date` がどの有効期間にも該当しない → `InputError::DateOutOfRange`
/// - Registry データの不整合 → `RegistryError` でパニック（起動時バリデーション）
pub fn load_brokerage_fee_params(target_date: LegalDate) -> Result<BrokerageFeeParams, JLawError> {
    let json_str = include_str!("../data/real_estate/brokerage_fee.json");

    let registry: BrokerageFeeRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::FileNotFound {
            path: format!("real_estate/brokerage_fee.json: {}", e),
        })?;

    let date_str = target_date.to_date_str();

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
            seller_only: s.seller_only,
        });

    // 不動産仲介報酬には軽減税率は適用されない（標準税率のみ）
    let consumption_tax = ConsumptionTaxParams {
        standard_rate: ConsumptionTaxRate {
            numer: entry.params.consumption_tax.numer,
            denom: entry.params.consumption_tax.denom,
        },
        reduced_rate: None,
    };

    BrokerageFeeParams {
        tiers,
        consumption_tax,
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
        let params = load_brokerage_fee_params(LegalDate::new(2024, 8, 1)).unwrap();
        assert_eq!(params.tiers.len(), 3);
        assert_eq!(params.consumption_tax.standard_rate.numer, 10);
        assert_eq!(params.consumption_tax.standard_rate.denom, 100);
        assert!(params.low_cost_special.is_some());
        let special = params.low_cost_special.unwrap();
        assert_eq!(special.price_ceiling_inclusive, 8_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 330_000);
    }

    #[test]
    fn load_2019_superseded_params() {
        let params = load_brokerage_fee_params(LegalDate::new(2019, 12, 1)).unwrap();
        assert_eq!(params.tiers.len(), 3);
        // 2019年告示は低廉特例あり（売主限定・400万円以下）
        let special = params.low_cost_special.as_ref().unwrap();
        assert_eq!(special.price_ceiling_inclusive, 4_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 180_000);
        assert!(special.seller_only);
    }

    #[test]
    fn date_out_of_range_returns_error() {
        // 2018年以前はカバー範囲外
        let result = load_brokerage_fee_params(LegalDate::new(2017, 12, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }

    #[test]
    fn boundary_2024_07_01_uses_new_params() {
        let params = load_brokerage_fee_params(LegalDate::new(2024, 7, 1)).unwrap();
        let special = params.low_cost_special.as_ref().unwrap();
        assert_eq!(special.price_ceiling_inclusive, 8_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 330_000);
        assert!(!special.seller_only);
    }

    #[test]
    fn boundary_2024_06_30_uses_old_params() {
        // 2024-06-30 は旧告示（売主限定・400万円以下の低廉特例）
        let params = load_brokerage_fee_params(LegalDate::new(2024, 6, 30)).unwrap();
        let special = params.low_cost_special.as_ref().unwrap();
        assert_eq!(special.price_ceiling_inclusive, 4_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 180_000);
        assert!(special.seller_only);
    }

    #[test]
    fn load_2018_params() {
        let params = load_brokerage_fee_params(LegalDate::new(2018, 6, 1)).unwrap();
        assert_eq!(params.tiers.len(), 3);
        assert_eq!(params.consumption_tax.standard_rate.numer, 8);
        assert_eq!(params.consumption_tax.standard_rate.denom, 100);
        let special = params.low_cost_special.as_ref().unwrap();
        assert_eq!(special.price_ceiling_inclusive, 4_000_000);
        assert_eq!(special.fee_ceiling_exclusive_tax, 180_000);
        assert!(special.seller_only);
    }

    #[test]
    fn boundary_2019_10_01_uses_new_tax_rate() {
        // 2019-10-01 から消費税10%
        let params = load_brokerage_fee_params(LegalDate::new(2019, 10, 1)).unwrap();
        assert_eq!(params.consumption_tax.standard_rate.numer, 10);
        // 低廉特例は引き続き売主限定・400万円以下
        let special = params.low_cost_special.as_ref().unwrap();
        assert_eq!(special.price_ceiling_inclusive, 4_000_000);
        assert!(special.seller_only);
    }

    #[test]
    fn boundary_2019_09_30_uses_8pct_tax() {
        // 2019-09-30 まで消費税8%
        let params = load_brokerage_fee_params(LegalDate::new(2019, 9, 30)).unwrap();
        assert_eq!(params.consumption_tax.standard_rate.numer, 8);
    }

    #[test]
    fn registry_validation_passes() {
        let json_str = include_str!("../data/real_estate/brokerage_fee.json");
        let registry: BrokerageFeeRegistry = serde_json::from_str(json_str).unwrap();
        validate(&registry).unwrap();
    }
}
