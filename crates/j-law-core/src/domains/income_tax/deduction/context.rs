use crate::types::date::LegalDate;
use std::fmt;

/// 所得控除計算の入力コンテキスト。
///
/// 総所得金額等から所得控除額を差し引き、課税所得金額を組み立てる。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
pub struct IncomeDeductionContext {
    /// 総所得金額等（円）。
    pub total_income_amount: u64,
    /// 計算対象年の基準日。
    pub target_date: LegalDate,
    /// 各所得控除の入力値。
    pub deductions: IncomeDeductionInput,
}

impl fmt::Debug for IncomeDeductionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncomeDeductionContext")
            .field("total_income_amount", &self.total_income_amount)
            .field("target_date", &self.target_date)
            .field("deductions", &self.deductions)
            .finish()
    }
}

/// 所得控除全体の入力集合。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
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
/// # 法的根拠
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone, Default)]
pub struct PersonalDeductionInput {
    /// 配偶者控除の入力。該当しない場合は `None`。
    pub spouse: Option<SpouseDeductionInput>,
    /// 扶養控除の入力。
    pub dependent: DependentDeductionInput,
}

/// 配偶者控除の入力。
///
/// # 法的根拠
/// 所得税法 第83条（配偶者控除）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpouseDeductionInput {
    /// 配偶者の合計所得金額（円）。
    pub spouse_total_income_amount: u64,
    /// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 納税者と配偶者が生計を一にすると呼び出し元が判定した場合に `true`。
    pub is_same_household: bool,
    /// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 老人控除対象配偶者に該当すると呼び出し元が判定した場合に `true`。
    pub is_elderly: bool,
}

/// 扶養控除の入力。
///
/// # 法的根拠
/// 所得税法 第84条（扶養控除）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DependentDeductionInput {
    /// WARNING: この人数の事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 一般の控除対象扶養親族の人数。
    pub general_count: u16,
    /// WARNING: この人数の事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 特定扶養親族の人数。
    pub specific_count: u16,
    /// WARNING: この人数の事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 同居老親等に該当する老人扶養親族の人数。
    pub elderly_cohabiting_count: u16,
    /// WARNING: この人数の事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    ///
    /// 同居老親等以外の老人扶養親族の人数。
    pub elderly_other_count: u16,
}

/// 支出系控除の入力。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
#[derive(Debug, Clone, Default)]
pub struct ExpenseDeductionInput {
    /// 社会保険料控除の対象として呼び出し元が判定した支払額（円）。
    pub social_insurance_premium_paid: u64,
    /// 医療費控除の入力。該当しない場合は `None`。
    pub medical: Option<MedicalDeductionInput>,
    /// 生命保険料控除の入力。該当しない場合は `None`。
    pub life_insurance: Option<LifeInsuranceDeductionInput>,
    /// 寄附金控除の入力。該当しない場合は `None`。
    pub donation: Option<DonationDeductionInput>,
}

/// 医療費控除の入力。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MedicalDeductionInput {
    /// その年中に支払った医療費の総額（円）。
    pub medical_expense_paid: u64,
    /// 保険金などで補填された金額（円）。
    pub reimbursed_amount: u64,
}

/// 生命保険料控除の入力。
///
/// # 法的根拠
/// 所得税法 第76条（生命保険料控除）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LifeInsuranceDeductionInput {
    /// 新契約の一般生命保険料の支払額（円）。
    pub new_general_paid_amount: u64,
    /// 新契約の個人年金保険料の支払額（円）。
    pub new_individual_pension_paid_amount: u64,
    /// 新契約の介護医療保険料の支払額（円）。
    pub new_care_medical_paid_amount: u64,
    /// 旧契約の一般生命保険料の支払額（円）。
    pub old_general_paid_amount: u64,
    /// 旧契約の個人年金保険料の支払額（円）。
    pub old_individual_pension_paid_amount: u64,
}

/// 寄附金控除の入力。
///
/// # 法的根拠
/// 所得税法 第78条（寄附金控除）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DonationDeductionInput {
    /// 寄附金控除の対象として呼び出し元が判定した寄附金支出額（円）。
    pub qualified_donation_amount: u64,
}
