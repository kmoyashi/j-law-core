use serde::Deserialize;

use crate::schema::Fraction;

#[derive(Debug, Clone, Deserialize)]
pub struct BasicDeductionBracketEntry {
    pub label: String,
    pub income_from: u64,
    pub income_to_inclusive: Option<u64>,
    pub deduction_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BasicDeductionParamsEntry {
    pub brackets: Vec<BasicDeductionBracketEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpouseIncomeBracketEntry {
    pub label: String,
    pub taxpayer_income_from: u64,
    pub taxpayer_income_to_inclusive: Option<u64>,
    pub deduction_amount: u64,
    pub elderly_deduction_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpouseDeductionParamsEntry {
    pub qualifying_spouse_income_max: u64,
    pub taxpayer_income_brackets: Vec<SpouseIncomeBracketEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DependentDeductionParamsEntry {
    pub general_deduction_amount: u64,
    pub specific_deduction_amount: u64,
    pub elderly_cohabiting_deduction_amount: u64,
    pub elderly_other_deduction_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonalDeductionParamsEntry {
    pub basic: BasicDeductionParamsEntry,
    pub spouse: SpouseDeductionParamsEntry,
    pub dependent: DependentDeductionParamsEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialInsuranceDeductionParamsEntry {}

#[derive(Debug, Clone, Deserialize)]
pub struct MedicalDeductionParamsEntry {
    pub income_threshold_rate: Fraction,
    pub threshold_cap_amount: u64,
    pub deduction_cap_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LifeInsuranceDeductionBracketEntry {
    pub label: String,
    pub paid_from: u64,
    pub paid_to_inclusive: Option<u64>,
    pub rate: Fraction,
    pub addition_amount: u64,
    pub deduction_cap_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LifeInsuranceDeductionParamsEntry {
    pub new_contract_brackets: Vec<LifeInsuranceDeductionBracketEntry>,
    pub old_contract_brackets: Vec<LifeInsuranceDeductionBracketEntry>,
    pub mixed_contract_cap_amount: u64,
    pub new_contract_cap_amount: u64,
    pub old_contract_cap_amount: u64,
    pub combined_cap_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DonationDeductionParamsEntry {
    pub income_cap_rate: Fraction,
    pub non_deductible_amount: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseDeductionParamsEntry {
    #[allow(dead_code)]
    pub social_insurance: SocialInsuranceDeductionParamsEntry,
    pub medical: MedicalDeductionParamsEntry,
    pub life_insurance: LifeInsuranceDeductionParamsEntry,
    pub donation: DonationDeductionParamsEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxDeductionParamsEntry {
    pub personal: PersonalDeductionParamsEntry,
    pub expense: ExpenseDeductionParamsEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxDeductionHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub params: IncomeTaxDeductionParamsEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxDeductionRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<IncomeTaxDeductionHistoryEntry>,
}
