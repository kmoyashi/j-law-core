use super::context::SocialInsurancePrefecture;

/// 社会保険料の率。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 厚生年金保険法 第81条
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocialInsuranceRate {
    pub numer: u64,
    pub denom: u64,
}

/// 都道府県別健康保険料率。
///
/// # 法的根拠
/// 健康保険法 第160条
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefectureHealthInsuranceRate {
    pub prefecture: SocialInsurancePrefecture,
    pub rate: SocialInsuranceRate,
}

/// 社会保険料計算パラメータ。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
#[derive(Debug, Clone)]
pub struct SocialInsuranceParams {
    /// 都道府県別の健康保険料率。
    pub prefecture_health_rates: Vec<PrefectureHealthInsuranceRate>,
    /// 介護保険料率。
    pub care_rate: SocialInsuranceRate,
    /// 厚生年金保険料率。
    pub pension_rate: SocialInsuranceRate,
    /// 健康保険の標準報酬月額として許容する値。
    pub valid_standard_monthly_remunerations: Vec<u64>,
    /// 厚生年金の標準報酬月額上限。
    pub pension_standard_monthly_remuneration_cap: u64,
}
