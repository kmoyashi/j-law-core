pub mod calculator;
pub mod context;
pub mod expense;
pub mod params;
pub mod personal;
pub mod types;

pub use calculator::calculate_income_deductions;
pub use context::{
    ExpenseDeductionInput, IncomeDeductionContext, IncomeDeductionInput, PersonalDeductionInput,
};
pub use params::{
    BasicDeductionBracket, BasicDeductionParams, ExpenseDeductionParams, IncomeDeductionParams,
    PersonalDeductionParams, SocialInsuranceDeductionParams,
};
pub use types::{IncomeDeductionKind, IncomeDeductionLine, IncomeDeductionResult};
