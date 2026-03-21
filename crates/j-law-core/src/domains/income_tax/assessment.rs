use std::collections::HashSet;
use std::fmt;

use crate::domains::income_tax::calculator::{calculate_income_tax_inner, IncomeTaxResult};
use crate::domains::income_tax::context::IncomeTaxFlag;
use crate::domains::income_tax::deduction::{
    calculate_income_deductions, IncomeDeductionContext, IncomeDeductionParams,
    IncomeDeductionResult,
};
use crate::domains::income_tax::params::IncomeTaxParams;
use crate::domains::income_tax::policy::IncomeTaxPolicy;
use crate::error::JLawError;

/// 所得控除の計算から所得税額の算出までを通しで実行するコンテキスト。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
/// 所得税法 第89条第1項（所得税の税率）
pub struct IncomeTaxAssessmentContext {
    /// 所得控除の入力コンテキスト。
    pub deduction_context: IncomeDeductionContext,
    /// 適用する法的フラグの集合。
    pub flags: HashSet<IncomeTaxFlag>,
    /// 計算ポリシー。
    pub policy: Box<dyn IncomeTaxPolicy>,
}

impl fmt::Debug for IncomeTaxAssessmentContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncomeTaxAssessmentContext")
            .field("deduction_context", &self.deduction_context)
            .field("flags", &self.flags)
            .field("policy", &"<policy>")
            .finish()
    }
}

/// 所得控除と所得税額の通し計算結果。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
/// 所得税法 第89条第1項（所得税の税率）
/// 復興財源確保法 第13条（復興特別所得税）
#[derive(Debug, Clone)]
pub struct IncomeTaxAssessmentResult {
    /// 所得控除の計算結果。
    pub deductions: IncomeDeductionResult,
    /// 課税所得金額に対する所得税額の計算結果。
    pub tax: IncomeTaxResult,
}

/// 所得控除の計算結果を使って所得税額を算出する。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
/// 所得税法 第89条第1項（所得税の税率）
/// 復興財源確保法 第13条（復興特別所得税）
pub fn calculate_income_tax_assessment(
    ctx: &IncomeTaxAssessmentContext,
    deduction_params: &IncomeDeductionParams,
    tax_params: &IncomeTaxParams,
) -> Result<IncomeTaxAssessmentResult, JLawError> {
    let deductions = calculate_income_deductions(&ctx.deduction_context, deduction_params)?;
    let tax = calculate_income_tax_inner(
        deductions.taxable_income.as_yen(),
        ctx.deduction_context.target_date,
        &ctx.flags,
        ctx.policy.as_ref(),
        tax_params,
    )?;

    Ok(IncomeTaxAssessmentResult { deductions, tax })
}
