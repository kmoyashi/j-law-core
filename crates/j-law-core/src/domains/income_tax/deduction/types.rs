use crate::types::amount::FinalAmount;

/// 所得控除の種別。
///
/// # 法的根拠
/// 所得税法 各条項
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomeDeductionKind {
    /// 基礎控除。
    Basic,
    /// 配偶者控除。
    Spouse,
    /// 扶養控除。
    Dependent,
    /// 社会保険料控除。
    SocialInsurance,
    /// 医療費控除。
    Medical,
    /// 生命保険料控除。
    LifeInsurance,
    /// 寄附金控除。
    Donation,
}

/// 所得控除の内訳1行。
///
/// # 法的根拠
/// 所得税法 各条項
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomeDeductionLine {
    /// 控除種別。
    pub kind: IncomeDeductionKind,
    /// 内訳表示用ラベル。
    pub label: String,
    /// 控除額（円）。
    pub amount: FinalAmount,
}

/// 所得控除の計算結果。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomeDeductionResult {
    /// 総所得金額等（円）。
    pub total_income_amount: FinalAmount,
    /// 所得控除額の合計（円）。
    pub total_deductions: FinalAmount,
    /// 1,000円未満切り捨て前の課税所得金額（円）。
    pub taxable_income_before_truncation: FinalAmount,
    /// 1,000円未満切り捨て後の課税所得金額（円）。
    pub taxable_income: FinalAmount,
    /// 所得控除の内訳。
    pub breakdown: Vec<IncomeDeductionLine>,
}
