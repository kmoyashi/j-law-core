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
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
#[derive(Debug, Clone)]
pub struct PersonalDeductionParams {
    /// 基礎控除のパラメータ。
    pub basic: BasicDeductionParams,
    /// 配偶者控除のパラメータ。
    pub spouse: SpouseDeductionParams,
    /// 扶養控除のパラメータ。
    pub dependent: DependentDeductionParams,
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

/// 配偶者控除のパラメータ。
///
/// # 法的根拠
/// 所得税法 第83条（配偶者控除）
#[derive(Debug, Clone)]
pub struct SpouseDeductionParams {
    /// 控除対象配偶者に該当する配偶者の合計所得金額上限（円）。
    pub qualifying_spouse_income_max: u64,
    /// 納税者本人の合計所得金額に応じた控除額テーブル。
    pub taxpayer_income_brackets: Vec<SpouseIncomeBracket>,
}

/// 配偶者控除の所得閾値テーブル。
///
/// # 法的根拠
/// 所得税法 第83条（配偶者控除）
#[derive(Debug, Clone)]
pub struct SpouseIncomeBracket {
    /// ブラケットの表示名。
    pub label: String,
    /// 納税者本人の合計所得金額の下限（円・この金額以上）。
    pub taxpayer_income_from: u64,
    /// 納税者本人の合計所得金額の上限（円・この金額以下）。`None` は上限なし。
    pub taxpayer_income_to_inclusive: Option<u64>,
    /// 一般の控除対象配偶者に適用する控除額（円）。
    pub deduction_amount: u64,
    /// 老人控除対象配偶者に適用する控除額（円）。
    pub elderly_deduction_amount: u64,
}

/// 扶養控除のパラメータ。
///
/// # 法的根拠
/// 所得税法 第84条（扶養控除）
#[derive(Debug, Clone, Copy)]
pub struct DependentDeductionParams {
    /// 一般の控除対象扶養親族1人当たりの控除額（円）。
    pub general_deduction_amount: u64,
    /// 特定扶養親族1人当たりの控除額（円）。
    pub specific_deduction_amount: u64,
    /// 同居老親等1人当たりの控除額（円）。
    pub elderly_cohabiting_deduction_amount: u64,
    /// 同居老親等以外の老人扶養親族1人当たりの控除額（円）。
    pub elderly_other_deduction_amount: u64,
}
