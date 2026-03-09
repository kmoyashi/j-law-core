/// 所得控除計算で使用するパラメータセット。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone)]
pub struct IncomeDeductionParams {
    /// 人的控除パラメータ。
    pub personal: PersonalDeductionParams,
    /// 支出系控除パラメータ。
    pub expense: ExpenseDeductionParams,
}

/// 人的控除パラメータ。
///
/// # 法的根拠
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone)]
pub struct PersonalDeductionParams {
    /// 基礎控除のパラメータ。
    pub basic: BasicDeductionParams,
}

/// 基礎控除のパラメータ。
///
/// # 法的根拠
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone)]
pub struct BasicDeductionParams {
    /// 総所得金額等に応じた基礎控除額テーブル。
    pub brackets: Vec<BasicDeductionBracket>,
}

/// 基礎控除の所得閾値テーブル。
///
/// # 法的根拠
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone)]
pub struct BasicDeductionBracket {
    /// ブラケットの表示名。
    pub label: String,
    /// 総所得金額等の下限（円・この金額以上）。
    pub income_from: u64,
    /// 総所得金額等の上限（円・この金額以下）。`None` は上限なし。
    pub income_to_inclusive: Option<u64>,
    /// 当該ブラケットで適用する基礎控除額（円）。
    pub deduction_amount: u64,
}

/// 支出系控除パラメータ。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
#[derive(Debug, Clone)]
pub struct ExpenseDeductionParams {
    /// 社会保険料控除のパラメータ。
    pub social_insurance: SocialInsuranceDeductionParams,
}

/// 社会保険料控除のパラメータ。
///
/// 現段階では全額控除のため追加設定を持たない。
///
/// # 法的根拠
/// 所得税法 第74条（社会保険料控除）
#[derive(Debug, Clone, Copy)]
pub struct SocialInsuranceDeductionParams;
