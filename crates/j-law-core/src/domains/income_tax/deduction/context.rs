use crate::types::date::LegalDate;

/// 所得控除計算の入力コンテキスト。
///
/// 総所得金額等から所得控除額を差し引き、課税所得金額を組み立てる。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
pub struct IncomeDeductionContext {
    /// 総所得金額等（円）。
    pub total_income_amount: u64,
    /// 計算対象年の基準日。
    pub target_date: LegalDate,
    /// 各所得控除の入力値。
    pub deductions: IncomeDeductionInput,
}

/// 所得控除全体の入力集合。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone, Default)]
pub struct IncomeDeductionInput {
    /// 人的控除の入力。
    pub personal: PersonalDeductionInput,
    /// 支出系控除の入力。
    pub expense: ExpenseDeductionInput,
}

/// 人的控除の入力。
///
/// 現段階では基礎控除のみを扱うため、追加入力は持たない。
///
/// # 法的根拠
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone, Default)]
pub struct PersonalDeductionInput {}

/// 支出系控除の入力。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
#[derive(Debug, Clone, Default)]
pub struct ExpenseDeductionInput {
    /// 社会保険料控除の対象として呼び出し元が判定した支払額（円）。
    pub social_insurance_premium_paid: u64,
}
