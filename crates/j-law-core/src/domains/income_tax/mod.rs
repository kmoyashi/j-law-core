pub mod assessment;
pub mod calculator;
pub mod context;
pub mod deduction;
pub mod params;
pub mod policy;

pub use assessment::{
    calculate_income_tax_assessment, IncomeTaxAssessmentContext, IncomeTaxAssessmentResult,
};
pub use calculator::{calculate_income_tax, IncomeTaxResult, IncomeTaxStep};
pub use context::{IncomeTaxContext, IncomeTaxFlag};
pub use deduction::{
    calculate_income_deductions, BasicDeductionBracket, BasicDeductionParams,
    ExpenseDeductionInput, ExpenseDeductionParams, IncomeDeductionContext, IncomeDeductionInput,
    IncomeDeductionKind, IncomeDeductionLine, IncomeDeductionParams, IncomeDeductionResult,
    PersonalDeductionInput, PersonalDeductionParams, SocialInsuranceDeductionParams,
};
pub use params::{IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams};
pub use policy::StandardIncomeTaxPolicy;
