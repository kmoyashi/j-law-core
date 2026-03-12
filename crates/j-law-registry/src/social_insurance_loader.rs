use crate::social_insurance_schema::{SocialInsuranceHistoryEntry, SocialInsuranceRegistry};
use j_law_core::domains::social_insurance::{
    PrefectureHealthInsuranceRate, SocialInsuranceParams, SocialInsurancePrefecture,
    SocialInsuranceRate,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `social_insurance.json` をロードして `target_date` に対応するパラメータを返す。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
pub fn load_social_insurance_params(
    target_date: LegalDate,
) -> Result<SocialInsuranceParams, JLawError> {
    let json_str = include_str!("../data/social_insurance/social_insurance.json");

    let registry: SocialInsuranceRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "social_insurance/social_insurance.json".into(),
            cause: e.to_string(),
        })?;

    validate_registry(&registry)?;

    let date_str = target_date.to_date_str();
    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    to_params(entry)
}

fn validate_registry(registry: &SocialInsuranceRegistry) -> Result<(), RegistryError> {
    for (entry_index, entry) in registry.history.iter().enumerate() {
        for (rate_index, rate) in entry.params.prefecture_health_rates.iter().enumerate() {
            if rate.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!(
                        "social_insurance/history[{entry_index}]/prefecture_health_rates[{rate_index}]/rate.denom"
                    ),
                });
            }
        }
        if entry.params.care_rate.denom == 0 {
            return Err(RegistryError::ZeroDenominator {
                path: format!("social_insurance/history[{entry_index}]/care_rate.denom"),
            });
        }
        if entry.params.pension_rate.denom == 0 {
            return Err(RegistryError::ZeroDenominator {
                path: format!("social_insurance/history[{entry_index}]/pension_rate.denom"),
            });
        }
    }

    Ok(())
}

fn find_entry<'a>(
    registry: &'a SocialInsuranceRegistry,
    date_str: &str,
) -> Option<&'a SocialInsuranceHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &SocialInsuranceHistoryEntry) -> Result<SocialInsuranceParams, JLawError> {
    let prefecture_health_rates = entry
        .params
        .prefecture_health_rates
        .iter()
        .map(|rate| {
            let prefecture = SocialInsurancePrefecture::from_code(rate.prefecture_code)
                .ok_or_else(|| RegistryError::ParseError {
                    path: "social_insurance/social_insurance.json".into(),
                    cause: format!("unknown prefecture_code: {}", rate.prefecture_code),
                })?;
            Ok(PrefectureHealthInsuranceRate {
                prefecture,
                rate: SocialInsuranceRate {
                    numer: rate.rate.numer,
                    denom: rate.rate.denom,
                },
            })
        })
        .collect::<Result<Vec<_>, RegistryError>>()?;

    Ok(SocialInsuranceParams {
        prefecture_health_rates,
        care_rate: SocialInsuranceRate {
            numer: entry.params.care_rate.numer,
            denom: entry.params.care_rate.denom,
        },
        pension_rate: SocialInsuranceRate {
            numer: entry.params.pension_rate.numer,
            denom: entry.params.pension_rate.denom,
        },
        valid_standard_monthly_remunerations: entry
            .params
            .valid_standard_monthly_remunerations
            .clone(),
        pension_standard_monthly_remuneration_cap: entry
            .params
            .pension_standard_monthly_remuneration_cap,
    })
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn load_2025_params() {
        let params = load_social_insurance_params(LegalDate::new(2025, 3, 1)).unwrap();
        let tokyo = params
            .prefecture_health_rates
            .iter()
            .find(|rate| rate.prefecture == SocialInsurancePrefecture::Tokyo)
            .unwrap();
        assert_eq!(tokyo.rate.numer, 991);
        assert_eq!(params.care_rate.numer, 159);
        assert_eq!(params.pension_rate.numer, 1_830);
    }

    #[test]
    fn load_2026_params() {
        let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
        let tokyo = params
            .prefecture_health_rates
            .iter()
            .find(|rate| rate.prefecture == SocialInsurancePrefecture::Tokyo)
            .unwrap();
        assert_eq!(tokyo.rate.numer, 985);
        assert_eq!(params.care_rate.numer, 162);
    }

    #[test]
    fn out_of_range_date_is_error() {
        let result = load_social_insurance_params(LegalDate::new(2025, 2, 28));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }
}
