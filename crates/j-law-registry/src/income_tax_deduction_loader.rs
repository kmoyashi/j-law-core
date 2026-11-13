use crate::income_tax_deduction_schema::{
    IncomeTaxDeductionHistoryEntry, IncomeTaxDeductionRegistry,
};
use j_law_core::domains::income_tax::deduction::{
    BasicDeductionBracket, BasicDeductionParams, DependentDeductionParams, DonationDeductionParams,
    ExpenseDeductionParams, IncomeDeductionParams, LifeInsuranceDeductionBracket,
    LifeInsuranceDeductionParams, MedicalDeductionParams, PersonalDeductionParams,
    SocialInsuranceDeductionParams, SpecificRelativeDeductionBracket,
    SpecificRelativeDeductionParams, SpouseDeductionParams, SpouseIncomeBracket,
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
/// 所得税法 第84条の2（特定親族特別控除）
/// 所得税法 第86条（基礎控除）
/// 租税特別措置法 第41条の16の2（基礎控除の特例）
pub fn load_income_tax_deduction_params(
    target_date: LegalDate,
) -> Result<IncomeDeductionParams, JLawError> {
    target_date.validate()?;

    let json_str = include_str!("../data/income_tax/deductions.json");

    let registry: IncomeTaxDeductionRegistry =
        serde_json::from_str(json_str).map_err(|e| RegistryError::ParseError {
            path: "income_tax/deductions.json".into(),
            cause: e.to_string(),
        })?;
    validate_registry(&registry)?;

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
            specific_relative: entry.params.personal.specific_relative.as_ref().map(
                |specific_relative| SpecificRelativeDeductionParams {
                    brackets: specific_relative
                        .brackets
                        .iter()
                        .map(|bracket| SpecificRelativeDeductionBracket {
                            label: bracket.label.clone(),
                            income_from: bracket.income_from,
                            income_to_inclusive: bracket.income_to_inclusive,
                            deduction_amount: bracket.deduction_amount,
                        })
                        .collect(),
                },
            ),
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

fn validate_registry(registry: &IncomeTaxDeductionRegistry) -> Result<(), JLawError> {
    validate_periods(registry)?;
    validate_denominators(registry)?;
    validate_bracket_tables(registry)?;
    Ok(())
}

/// 所得区分テーブルが空白・重複なく連続しているかを検証する。
///
/// 特定親族特別控除の下限は扶養親族等の所得要件（配偶者控除の所得要件と同額）の
/// 直後でなければならない。所得要件の改正時に片方だけ更新する誤りを検出する。
fn validate_bracket_tables(registry: &IncomeTaxDeductionRegistry) -> Result<(), RegistryError> {
    for (entry_index, entry) in registry.history.iter().enumerate() {
        let personal = &entry.params.personal;
        let path =
            |name: &str| format!("{}/history[{entry_index}]/personal/{name}", registry.domain);

        validate_contiguous_ranges(
            &path("basic/brackets"),
            personal
                .basic
                .brackets
                .iter()
                .map(|b| (b.income_from, b.income_to_inclusive)),
            0,
            true,
        )?;
        validate_contiguous_ranges(
            &path("spouse/taxpayer_income_brackets"),
            personal
                .spouse
                .taxpayer_income_brackets
                .iter()
                .map(|b| (b.taxpayer_income_from, b.taxpayer_income_to_inclusive)),
            0,
            true,
        )?;

        if let Some(specific_relative) = &personal.specific_relative {
            let path = path("specific_relative/brackets");
            let expected_start = personal
                .spouse
                .qualifying_spouse_income_max
                .checked_add(1)
                .ok_or_else(|| RegistryError::InvalidBracketTable {
                    path: path.clone(),
                    reason: "qualifying_spouse_income_max が上限値です".into(),
                })?;
            validate_contiguous_ranges(
                &path,
                specific_relative
                    .brackets
                    .iter()
                    .map(|b| (b.income_from, b.income_to_inclusive)),
                expected_start,
                false,
            )?;
        }
    }

    Ok(())
}

fn validate_contiguous_ranges(
    path: &str,
    ranges: impl Iterator<Item = (u64, Option<u64>)>,
    expected_start: u64,
    last_must_be_open_ended: bool,
) -> Result<(), RegistryError> {
    let invalid = |reason: String| RegistryError::InvalidBracketTable {
        path: path.into(),
        reason,
    };

    let mut next_from = Some(expected_start);
    let mut count = 0_usize;
    for (index, (from, to_inclusive)) in ranges.enumerate() {
        count += 1;
        let Some(expected_from) = next_from else {
            return Err(invalid(format!(
                "[{index}] 上限なしの区分の後に区分があります"
            )));
        };
        if from != expected_from {
            return Err(invalid(format!(
                "[{index}] income_from={from} は {expected_from} であるべきです（区分の空白または重複）"
            )));
        }
        next_from = match to_inclusive {
            Some(to) if to < from => {
                return Err(invalid(format!(
                    "[{index}] 上限 {to} が下限 {from} を下回っています"
                )));
            }
            Some(to) => Some(
                to.checked_add(1)
                    .ok_or_else(|| invalid(format!("[{index}] 上限が u64 の最大値です")))?,
            ),
            None => None,
        };
    }

    if count == 0 {
        return Err(invalid("区分が空です".into()));
    }
    match (last_must_be_open_ended, next_from) {
        (true, Some(_)) => Err(invalid(
            "最後の区分は上限なし（null）である必要があります".into(),
        )),
        (false, None) => Err(invalid("最後の区分には上限が必要です".into())),
        _ => Ok(()),
    }
}

fn validate_periods(registry: &IncomeTaxDeductionRegistry) -> Result<(), RegistryError> {
    let mut sorted = registry.history.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.effective_from.cmp(&b.effective_from));

    for [current, next] in sorted.array_windows::<2>() {
        let current = *current;
        let next = *next;

        let Some(current_until) = &current.effective_until else {
            return Err(RegistryError::PeriodOverlap {
                domain: registry.domain.clone(),
                from: next.effective_from.clone(),
                until: "open-ended".into(),
            });
        };

        if current_until >= &next.effective_from {
            return Err(RegistryError::PeriodOverlap {
                domain: registry.domain.clone(),
                from: next.effective_from.clone(),
                until: current_until.clone(),
            });
        }

        let expected_next_start =
            next_date_str(current_until, "income_tax/deductions.json/effective_until")?;
        if expected_next_start != next.effective_from {
            return Err(RegistryError::PeriodGap {
                domain: registry.domain.clone(),
                end: current_until.clone(),
                next_start: next.effective_from.clone(),
            });
        }
    }

    Ok(())
}

fn validate_denominators(registry: &IncomeTaxDeductionRegistry) -> Result<(), RegistryError> {
    for (entry_index, entry) in registry.history.iter().enumerate() {
        if entry.params.expense.medical.income_threshold_rate.denom == 0 {
            return Err(RegistryError::ZeroDenominator {
                path: format!(
                    "{}/history[{entry_index}]/expense/medical/income_threshold_rate.denom",
                    registry.domain
                ),
            });
        }
        if entry.params.expense.donation.income_cap_rate.denom == 0 {
            return Err(RegistryError::ZeroDenominator {
                path: format!(
                    "{}/history[{entry_index}]/expense/donation/income_cap_rate.denom",
                    registry.domain
                ),
            });
        }

        for (bracket_index, bracket) in entry
            .params
            .expense
            .life_insurance
            .new_contract_brackets
            .iter()
            .enumerate()
        {
            if bracket.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!(
                        "{}/history[{entry_index}]/expense/life_insurance/new_contract_brackets[{bracket_index}]/rate.denom",
                        registry.domain
                    ),
                });
            }
        }

        for (bracket_index, bracket) in entry
            .params
            .expense
            .life_insurance
            .old_contract_brackets
            .iter()
            .enumerate()
        {
            if bracket.rate.denom == 0 {
                return Err(RegistryError::ZeroDenominator {
                    path: format!(
                        "{}/history[{entry_index}]/expense/life_insurance/old_contract_brackets[{bracket_index}]/rate.denom",
                        registry.domain
                    ),
                });
            }
        }
    }

    Ok(())
}

fn next_date_str(date: &str, path: &str) -> Result<String, RegistryError> {
    let (year, month, day) = parse_iso_date(date, path)?;
    let max_day = days_in_month(year, month);

    let (next_year, next_month, next_day) = if day < max_day {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    };

    Ok(format!("{next_year:04}-{next_month:02}-{next_day:02}"))
}

fn parse_iso_date(date: &str, path: &str) -> Result<(u32, u32, u32), RegistryError> {
    let mut parts = date.split('-');
    let year = parts
        .next()
        .ok_or_else(|| invalid_date_parse_error(path, date))?
        .parse::<u32>()
        .map_err(|_| invalid_date_parse_error(path, date))?;
    let month = parts
        .next()
        .ok_or_else(|| invalid_date_parse_error(path, date))?
        .parse::<u32>()
        .map_err(|_| invalid_date_parse_error(path, date))?;
    let day = parts
        .next()
        .ok_or_else(|| invalid_date_parse_error(path, date))?
        .parse::<u32>()
        .map_err(|_| invalid_date_parse_error(path, date))?;

    if parts.next().is_some()
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
    {
        return Err(invalid_date_parse_error(path, date));
    }

    Ok((year, month, day))
}

fn invalid_date_parse_error(path: &str, value: &str) -> RegistryError {
    RegistryError::ParseError {
        path: path.into(),
        cause: format!("invalid ISO date: {value}"),
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use serde_json::json;

    fn registry_from_value(value: serde_json::Value) -> IncomeTaxDeductionRegistry {
        serde_json::from_value(value).unwrap()
    }

    fn sample_registry() -> IncomeTaxDeductionRegistry {
        registry_from_value(json!({
            "domain": "income_tax",
            "history": [
                {
                    "effective_from": "2019-01-01",
                    "effective_until": "2019-12-31",
                    "params": {
                        "personal": {
                            "basic": {
                                "brackets": [
                                    {
                                        "label": "一律",
                                        "income_from": 0,
                                        "income_to_inclusive": null,
                                        "deduction_amount": 380000
                                    }
                                ]
                            },
                            "spouse": {
                                "qualifying_spouse_income_max": 380000,
                                "taxpayer_income_brackets": [
                                    {
                                        "label": "1000万円以下",
                                        "taxpayer_income_from": 0,
                                        "taxpayer_income_to_inclusive": null,
                                        "deduction_amount": 380000,
                                        "elderly_deduction_amount": 480000
                                    }
                                ]
                            },
                            "dependent": {
                                "general_deduction_amount": 380000,
                                "specific_deduction_amount": 630000,
                                "elderly_cohabiting_deduction_amount": 580000,
                                "elderly_other_deduction_amount": 480000
                            }
                        },
                        "expense": {
                            "social_insurance": {},
                            "medical": {
                                "income_threshold_rate": { "numer": 5, "denom": 100 },
                                "threshold_cap_amount": 100000,
                                "deduction_cap_amount": 2000000
                            },
                            "life_insurance": {
                                "new_contract_brackets": [
                                    {
                                        "label": "一律",
                                        "paid_from": 0,
                                        "paid_to_inclusive": null,
                                        "rate": { "numer": 1, "denom": 1 },
                                        "addition_amount": 0,
                                        "deduction_cap_amount": 40000
                                    }
                                ],
                                "old_contract_brackets": [
                                    {
                                        "label": "一律",
                                        "paid_from": 0,
                                        "paid_to_inclusive": null,
                                        "rate": { "numer": 1, "denom": 1 },
                                        "addition_amount": 0,
                                        "deduction_cap_amount": 50000
                                    }
                                ],
                                "mixed_contract_cap_amount": 40000,
                                "new_contract_cap_amount": 40000,
                                "old_contract_cap_amount": 50000,
                                "combined_cap_amount": 120000
                            },
                            "donation": {
                                "income_cap_rate": { "numer": 40, "denom": 100 },
                                "non_deductible_amount": 2000
                            }
                        }
                    }
                },
                {
                    "effective_from": "2020-01-01",
                    "effective_until": null,
                    "params": {
                        "personal": {
                            "basic": {
                                "brackets": [
                                    {
                                        "label": "一律",
                                        "income_from": 0,
                                        "income_to_inclusive": null,
                                        "deduction_amount": 480000
                                    }
                                ]
                            },
                            "spouse": {
                                "qualifying_spouse_income_max": 480000,
                                "taxpayer_income_brackets": [
                                    {
                                        "label": "1000万円以下",
                                        "taxpayer_income_from": 0,
                                        "taxpayer_income_to_inclusive": null,
                                        "deduction_amount": 380000,
                                        "elderly_deduction_amount": 480000
                                    }
                                ]
                            },
                            "dependent": {
                                "general_deduction_amount": 380000,
                                "specific_deduction_amount": 630000,
                                "elderly_cohabiting_deduction_amount": 580000,
                                "elderly_other_deduction_amount": 480000
                            }
                        },
                        "expense": {
                            "social_insurance": {},
                            "medical": {
                                "income_threshold_rate": { "numer": 5, "denom": 100 },
                                "threshold_cap_amount": 100000,
                                "deduction_cap_amount": 2000000
                            },
                            "life_insurance": {
                                "new_contract_brackets": [
                                    {
                                        "label": "一律",
                                        "paid_from": 0,
                                        "paid_to_inclusive": null,
                                        "rate": { "numer": 1, "denom": 1 },
                                        "addition_amount": 0,
                                        "deduction_cap_amount": 40000
                                    }
                                ],
                                "old_contract_brackets": [
                                    {
                                        "label": "一律",
                                        "paid_from": 0,
                                        "paid_to_inclusive": null,
                                        "rate": { "numer": 1, "denom": 1 },
                                        "addition_amount": 0,
                                        "deduction_cap_amount": 50000
                                    }
                                ],
                                "mixed_contract_cap_amount": 40000,
                                "new_contract_cap_amount": 40000,
                                "old_contract_cap_amount": 50000,
                                "combined_cap_amount": 120000
                            },
                            "donation": {
                                "income_cap_rate": { "numer": 40, "denom": 100 },
                                "non_deductible_amount": 2000
                            }
                        }
                    }
                }
            ]
        }))
    }

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
    fn load_2024_params_has_no_specific_relative_deduction() {
        let params = load_income_tax_deduction_params(LegalDate::new(2024, 12, 31)).unwrap();
        assert_eq!(params.personal.basic.brackets[0].deduction_amount, 480_000);
        assert!(params.personal.specific_relative.is_none());
    }

    /// 令和7年分（国税庁 タックスアンサー No.1199 / No.1177 / No.1191）。
    #[test]
    fn load_2025_params() {
        let params = load_income_tax_deduction_params(LegalDate::new(2025, 1, 1)).unwrap();
        let amounts = params
            .personal
            .basic
            .brackets
            .iter()
            .map(|b| (b.income_to_inclusive, b.deduction_amount))
            .collect::<Vec<_>>();
        assert_eq!(
            amounts,
            vec![
                (Some(1_320_000), 950_000),
                (Some(3_360_000), 880_000),
                (Some(4_890_000), 680_000),
                (Some(6_550_000), 630_000),
                (Some(23_500_000), 580_000),
                (Some(24_000_000), 480_000),
                (Some(24_500_000), 320_000),
                (Some(25_000_000), 160_000),
                (None, 0),
            ]
        );
        assert_eq!(params.personal.spouse.qualifying_spouse_income_max, 580_000);
        let specific_relative = params.personal.specific_relative.unwrap();
        assert_eq!(specific_relative.brackets[0].income_from, 580_001);
        assert_eq!(specific_relative.brackets[0].deduction_amount, 630_000);
    }

    /// 令和8年分・令和9年分（国税庁「源泉所得税の改正のあらまし」令和8年4月）。
    #[test]
    fn load_2026_and_2027_params() {
        for date in [LegalDate::new(2026, 1, 1), LegalDate::new(2027, 12, 31)] {
            let params = load_income_tax_deduction_params(date).unwrap();
            let amounts = params
                .personal
                .basic
                .brackets
                .iter()
                .map(|b| (b.income_to_inclusive, b.deduction_amount))
                .collect::<Vec<_>>();
            assert_eq!(
                amounts,
                vec![
                    (Some(4_890_000), 1_040_000),
                    (Some(6_550_000), 670_000),
                    (Some(23_500_000), 620_000),
                    (Some(24_000_000), 480_000),
                    (Some(24_500_000), 320_000),
                    (Some(25_000_000), 160_000),
                    (None, 0),
                ]
            );
            assert_eq!(params.personal.spouse.qualifying_spouse_income_max, 620_000);
            let specific_relative = params.personal.specific_relative.unwrap();
            assert_eq!(specific_relative.brackets[0].income_from, 620_001);
        }
    }

    /// 令和10年分以後（国税庁「源泉所得税の改正のあらまし」令和8年4月）。
    #[test]
    fn load_2028_params() {
        let params = load_income_tax_deduction_params(LegalDate::new(2028, 1, 1)).unwrap();
        let amounts = params
            .personal
            .basic
            .brackets
            .iter()
            .map(|b| (b.income_to_inclusive, b.deduction_amount))
            .collect::<Vec<_>>();
        assert_eq!(
            amounts,
            vec![
                (Some(1_320_000), 990_000),
                (Some(23_500_000), 620_000),
                (Some(24_000_000), 480_000),
                (Some(24_500_000), 320_000),
                (Some(25_000_000), 160_000),
                (None, 0),
            ]
        );
        assert_eq!(params.personal.spouse.qualifying_spouse_income_max, 620_000);
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

    #[test]
    fn registry_validation_accepts_current_data() {
        let json_str = include_str!("../data/income_tax/deductions.json");
        let registry: IncomeTaxDeductionRegistry = serde_json::from_str(json_str).unwrap();
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_rejects_period_overlap() {
        let mut registry = sample_registry();
        registry.history[0].effective_until = Some("2020-01-15".into());

        let result = validate_registry(&registry);
        assert!(matches!(
            result,
            Err(JLawError::Registry(RegistryError::PeriodOverlap { .. }))
        ));
    }

    #[test]
    fn registry_validation_rejects_period_gap() {
        let mut registry = sample_registry();
        registry.history[0].effective_until = Some("2019-12-30".into());

        let result = validate_registry(&registry);
        assert!(matches!(
            result,
            Err(JLawError::Registry(RegistryError::PeriodGap { .. }))
        ));
    }

    #[test]
    fn registry_validation_rejects_zero_denominator() {
        let mut registry = sample_registry();
        registry.history[0]
            .params
            .expense
            .medical
            .income_threshold_rate
            .denom = 0;

        let result = validate_registry(&registry);
        assert!(matches!(
            result,
            Err(JLawError::Registry(RegistryError::ZeroDenominator { .. }))
        ));
    }

    fn invalid_bracket(result: Result<(), JLawError>) -> bool {
        matches!(
            result,
            Err(JLawError::Registry(
                RegistryError::InvalidBracketTable { .. }
            ))
        )
    }

    fn two_basic_brackets(
        first_to: u64,
        second_from: u64,
    ) -> Vec<crate::income_tax_deduction_schema::BasicDeductionBracketEntry> {
        serde_json::from_value(json!([
            { "label": "a", "income_from": 0, "income_to_inclusive": first_to, "deduction_amount": 1 },
            { "label": "b", "income_from": second_from, "income_to_inclusive": null, "deduction_amount": 0 }
        ]))
        .unwrap()
    }

    #[test]
    fn registry_validation_accepts_contiguous_brackets() {
        let mut registry = sample_registry();
        registry.history[0].params.personal.basic.brackets = two_basic_brackets(100, 101);
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_rejects_bracket_gap() {
        let mut registry = sample_registry();
        registry.history[0].params.personal.basic.brackets = two_basic_brackets(100, 102);
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    #[test]
    fn registry_validation_rejects_bracket_overlap() {
        let mut registry = sample_registry();
        registry.history[0].params.personal.basic.brackets = two_basic_brackets(100, 100);
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    #[test]
    fn registry_validation_rejects_bounded_last_basic_bracket() {
        let mut registry = sample_registry();
        registry.history[0].params.personal.basic.brackets[0].income_to_inclusive = Some(100);
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    #[test]
    fn registry_validation_rejects_basic_brackets_not_starting_at_zero() {
        let mut registry = sample_registry();
        registry.history[0].params.personal.basic.brackets[0].income_from = 1;
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    fn specific_relative(
        from: u64,
        to: Option<u64>,
    ) -> crate::income_tax_deduction_schema::SpecificRelativeDeductionParamsEntry {
        serde_json::from_value(json!({
            "brackets": [
                { "label": "a", "income_from": from, "income_to_inclusive": to, "deduction_amount": 630000 }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn registry_validation_accepts_specific_relative_aligned_with_income_requirement() {
        let mut registry = sample_registry();
        registry.history[1].params.personal.specific_relative =
            Some(specific_relative(480_001, Some(1_230_000)));
        assert!(validate_registry(&registry).is_ok());
    }

    #[test]
    fn registry_validation_rejects_specific_relative_misaligned_with_income_requirement() {
        // 扶養親族の所得要件（配偶者の所得要件と同額）を更新し忘れたケースを検出する。
        let mut registry = sample_registry();
        registry.history[1].params.personal.specific_relative =
            Some(specific_relative(580_001, Some(1_230_000)));
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    #[test]
    fn registry_validation_rejects_open_ended_specific_relative() {
        let mut registry = sample_registry();
        registry.history[1].params.personal.specific_relative =
            Some(specific_relative(480_001, None));
        assert!(invalid_bracket(validate_registry(&registry)));
    }

    #[test]
    fn parse_iso_date_accepts_leap_day_in_leap_year() {
        let result = parse_iso_date("2024-02-29", "history[0].effective_from");
        assert!(matches!(result, Ok((2024, 2, 29))));
    }

    #[test]
    fn parse_iso_date_rejects_leap_day_in_common_year() {
        let result = parse_iso_date("2023-02-29", "history[0].effective_from");
        assert!(matches!(result, Err(RegistryError::ParseError { .. })));
    }

    #[test]
    fn parse_iso_date_rejects_century_non_leap_year() {
        let result = parse_iso_date("1900-02-29", "history[0].effective_from");
        assert!(matches!(result, Err(RegistryError::ParseError { .. })));
    }

    #[test]
    fn parse_iso_date_accepts_century_leap_year() {
        let result = parse_iso_date("2000-02-29", "history[0].effective_from");
        assert!(matches!(result, Ok((2000, 2, 29))));
    }
}
