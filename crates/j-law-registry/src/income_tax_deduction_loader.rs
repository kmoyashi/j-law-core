use crate::income_tax_deduction_schema::{
    IncomeTaxDeductionHistoryEntry, IncomeTaxDeductionRegistry,
};
use j_law_core::domains::income_tax::deduction::{
    BasicDeductionBracket, BasicDeductionParams, DependentDeductionParams, DonationDeductionParams,
    ExpenseDeductionParams, IncomeDeductionParams, LifeInsuranceDeductionBracket,
    LifeInsuranceDeductionParams, MedicalDeductionParams, PersonalDeductionParams,
    SocialInsuranceDeductionParams, SpouseDeductionParams, SpouseIncomeBracket,
};
use j_law_core::types::date::LegalDate;
use j_law_core::{InputError, JLawError, RegistryError};

/// `deductions.json` をロードして `target_date` に対応する所得控除パラメータを返す。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
pub fn load_income_tax_deduction_params(
    target_date: LegalDate,
) -> Result<IncomeDeductionParams, JLawError> {
    let json_str = include_str!("../data/income_tax/deductions.json");

    let registry: IncomeTaxDeductionRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "income_tax/deductions.json".into(),
            cause: e.to_string(),
        })?;

    let date_str = target_date.to_date_str();
    let entry = find_entry(&registry, &date_str).ok_or_else(|| InputError::DateOutOfRange {
        date: date_str.clone(),
    })?;

    Ok(to_params(entry))
}

fn find_entry<'a>(
    registry: &'a IncomeTaxDeductionRegistry,
    date_str: &str,
) -> Option<&'a IncomeTaxDeductionHistoryEntry> {
    registry.history.iter().find(|entry| {
        let from_ok = entry.effective_from.as_str() <= date_str;
        let until_ok = match &entry.effective_until {
            Some(until) => date_str <= until.as_str(),
            None => true,
        };
        from_ok && until_ok
    })
}

fn to_params(entry: &IncomeTaxDeductionHistoryEntry) -> IncomeDeductionParams {
    IncomeDeductionParams {
        personal: PersonalDeductionParams {
            basic: BasicDeductionParams {
                brackets: entry
                    .params
                    .personal
                    .basic
                    .brackets
                    .iter()
                    .map(|bracket| BasicDeductionBracket {
                        label: bracket.label.clone(),
                        income_from: bracket.income_from,
                        income_to_inclusive: bracket.income_to_inclusive,
                        deduction_amount: bracket.deduction_amount,
                    })
                    .collect(),
            },
            spouse: SpouseDeductionParams {
                qualifying_spouse_income_max: entry
                    .params
                    .personal
                    .spouse
                    .qualifying_spouse_income_max,
                taxpayer_income_brackets: entry
                    .params
                    .personal
                    .spouse
                    .taxpayer_income_brackets
                    .iter()
                    .map(|bracket| SpouseIncomeBracket {
                        label: bracket.label.clone(),
                        taxpayer_income_from: bracket.taxpayer_income_from,
                        taxpayer_income_to_inclusive: bracket.taxpayer_income_to_inclusive,
                        deduction_amount: bracket.deduction_amount,
                        elderly_deduction_amount: bracket.elderly_deduction_amount,
                    })
                    .collect(),
            },
            dependent: DependentDeductionParams {
                general_deduction_amount: entry.params.personal.dependent.general_deduction_amount,
                specific_deduction_amount: entry
                    .params
                    .personal
                    .dependent
                    .specific_deduction_amount,
                elderly_cohabiting_deduction_amount: entry
                    .params
                    .personal
                    .dependent
                    .elderly_cohabiting_deduction_amount,
                elderly_other_deduction_amount: entry
                    .params
                    .personal
                    .dependent
                    .elderly_other_deduction_amount,
            },
        },
        expense: ExpenseDeductionParams {
            social_insurance: SocialInsuranceDeductionParams,
            medical: MedicalDeductionParams {
                income_threshold_rate_numer: entry
                    .params
                    .expense
                    .medical
                    .income_threshold_rate
                    .numer,
                income_threshold_rate_denom: entry
                    .params
                    .expense
                    .medical
                    .income_threshold_rate
                    .denom,
                threshold_cap_amount: entry.params.expense.medical.threshold_cap_amount,
                deduction_cap_amount: entry.params.expense.medical.deduction_cap_amount,
            },
            life_insurance: LifeInsuranceDeductionParams {
                new_contract_brackets: entry
                    .params
                    .expense
                    .life_insurance
                    .new_contract_brackets
                    .iter()
                    .map(to_life_insurance_bracket)
                    .collect(),
                old_contract_brackets: entry
                    .params
                    .expense
                    .life_insurance
                    .old_contract_brackets
                    .iter()
                    .map(to_life_insurance_bracket)
                    .collect(),
                mixed_contract_cap_amount: entry
                    .params
                    .expense
                    .life_insurance
                    .mixed_contract_cap_amount,
                new_contract_cap_amount: entry
                    .params
                    .expense
                    .life_insurance
                    .new_contract_cap_amount,
                old_contract_cap_amount: entry
                    .params
                    .expense
                    .life_insurance
                    .old_contract_cap_amount,
                combined_cap_amount: entry.params.expense.life_insurance.combined_cap_amount,
            },
            donation: DonationDeductionParams {
                income_cap_rate_numer: entry.params.expense.donation.income_cap_rate.numer,
                income_cap_rate_denom: entry.params.expense.donation.income_cap_rate.denom,
                non_deductible_amount: entry.params.expense.donation.non_deductible_amount,
            },
        },
    }
}

fn to_life_insurance_bracket(
    bracket: &crate::income_tax_deduction_schema::LifeInsuranceDeductionBracketEntry,
) -> LifeInsuranceDeductionBracket {
    LifeInsuranceDeductionBracket {
        label: bracket.label.clone(),
        paid_from: bracket.paid_from,
        paid_to_inclusive: bracket.paid_to_inclusive,
        rate_numer: bracket.rate.numer,
        rate_denom: bracket.rate.denom,
        addition_amount: bracket.addition_amount,
        deduction_cap_amount: bracket.deduction_cap_amount,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn load_2024_params() {
        let params = load_income_tax_deduction_params(LegalDate::new(2024, 1, 1)).unwrap();
        assert_eq!(params.personal.basic.brackets[0].deduction_amount, 480_000);
        assert_eq!(params.personal.spouse.qualifying_spouse_income_max, 480_000);
        assert_eq!(params.expense.medical.deduction_cap_amount, 2_000_000);
        assert_eq!(params.expense.life_insurance.combined_cap_amount, 120_000);
        assert_eq!(params.expense.donation.non_deductible_amount, 2_000);
    }

    #[test]
    fn load_2019_params() {
        let params = load_income_tax_deduction_params(LegalDate::new(2019, 6, 1)).unwrap();
        assert_eq!(params.personal.basic.brackets.len(), 1);
        assert_eq!(params.personal.basic.brackets[0].deduction_amount, 380_000);
        assert_eq!(params.personal.spouse.qualifying_spouse_income_max, 380_000);
    }

    #[test]
    fn boundary_2019_12_31_is_pre_2020() {
        let params = load_income_tax_deduction_params(LegalDate::new(2019, 12, 31)).unwrap();
        assert_eq!(params.personal.basic.brackets[0].deduction_amount, 380_000);
    }

    #[test]
    fn boundary_2020_01_01_is_post_2020() {
        let params = load_income_tax_deduction_params(LegalDate::new(2020, 1, 1)).unwrap();
        assert_eq!(params.personal.basic.brackets[0].deduction_amount, 480_000);
    }

    #[test]
    fn date_out_of_range_returns_error() {
        let result = load_income_tax_deduction_params(LegalDate::new(2017, 12, 31));
        assert!(matches!(
            result,
            Err(JLawError::Input(InputError::DateOutOfRange { .. }))
        ));
    }
}
