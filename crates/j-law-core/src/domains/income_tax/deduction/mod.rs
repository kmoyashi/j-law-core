pub mod calculator;
pub mod context;
pub mod expense;
pub mod params;
pub mod personal;
pub mod types;

pub use calculator::calculate_income_deductions;
pub use context::{
    DependentDeductionInput, DonationDeductionInput, ExpenseDeductionInput, IncomeDeductionContext,
    IncomeDeductionInput, LifeInsuranceDeductionInput, MedicalDeductionInput,
    PersonalDeductionInput, SpouseDeductionInput,
};
pub use params::{
    BasicDeductionBracket, BasicDeductionParams, DependentDeductionParams, DonationDeductionParams,
    ExpenseDeductionParams, IncomeDeductionParams, LifeInsuranceDeductionBracket,
    LifeInsuranceDeductionParams, MedicalDeductionParams, PersonalDeductionParams,
    SocialInsuranceDeductionParams, SpouseDeductionParams, SpouseIncomeBracket,
};
pub use types::{IncomeDeductionKind, IncomeDeductionLine, IncomeDeductionResult};
