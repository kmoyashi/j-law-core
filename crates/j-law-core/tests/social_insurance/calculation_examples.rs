//! 社会保険料ドメインの公式計算例テスト。
//!
//! 出典:
//! - 協会けんぽ東京支部「令和8年3月分（4月納付分）からの健康保険・厚生年金保険の保険料額表」
//! - 協会けんぽ神奈川支部「令和8年3月分（4月納付分）からの健康保険・厚生年金保険の保険料額表」
//! - 協会けんぽ沖縄支部「令和8年3月分（4月納付分）からの健康保険・厚生年金保険の保険料額表」

#![allow(clippy::disallowed_methods)]

use std::collections::HashSet;

use j_law_core::domains::social_insurance::{
    calculate_social_insurance_premium, SocialInsuranceContext, SocialInsuranceFlag,
    SocialInsurancePrefecture, StandardNenkinPolicy,
};
use j_law_core::LegalDate;
use j_law_registry::load_social_insurance_params;

fn ctx(
    standard_monthly_remuneration: u64,
    prefecture: SocialInsurancePrefecture,
    care: bool,
) -> SocialInsuranceContext {
    let mut flags = HashSet::new();
    if care {
        flags.insert(SocialInsuranceFlag::IsCareInsuranceApplicable);
    }
    SocialInsuranceContext {
        standard_monthly_remuneration,
        target_date: LegalDate::new(2026, 3, 1),
        prefecture,
        flags,
        policy: Box::new(StandardNenkinPolicy),
    }
}

/// 東京支部・標準報酬月額118,000円・介護なし。
///
/// 2026年3月分表では健康保険料の本人負担額は 5,811.5 円、
/// 厚生年金保険料の本人負担額は 10,797 円。
/// 給与控除では 50 銭以下切り捨てなので健康保険料は 5,811 円となる。
#[test]
fn tokyo_118k_without_care() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(118_000, SocialInsurancePrefecture::Tokyo, false),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 5_811);
    assert_eq!(result.pension_amount.as_yen(), 10_797);
    assert_eq!(result.total_amount.as_yen(), 16_608);
}

/// 東京支部・標準報酬月額118,000円・介護あり。
///
/// 2026年3月分表では健康保険料・介護保険料合計の本人負担額は 6,767.3 円。
#[test]
fn tokyo_118k_with_care() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(118_000, SocialInsurancePrefecture::Tokyo, true),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 6_767);
    assert_eq!(result.pension_amount.as_yen(), 10_797);
    assert_eq!(result.total_amount.as_yen(), 17_564);
}

/// 神奈川支部・標準報酬月額160,000円・介護なし。
///
/// 2026年3月分表では健康保険料の本人負担額は 7,936 円、
/// 厚生年金保険料の本人負担額は 14,640 円。
#[test]
fn kanagawa_160k_without_care() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(160_000, SocialInsurancePrefecture::Kanagawa, false),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 7_936);
    assert_eq!(result.pension_amount.as_yen(), 14_640);
    assert_eq!(result.total_amount.as_yen(), 22_576);
}

/// 沖縄支部・標準報酬月額98,000円・介護なし。
///
/// 2026年3月分表では健康保険料の本人負担額は 4,625.6 円、
/// 厚生年金保険料の本人負担額は 8,967 円。
#[test]
fn okinawa_98k_without_care() {
    let params = load_social_insurance_params(LegalDate::new(2026, 3, 1)).unwrap();
    let result = calculate_social_insurance_premium(
        &ctx(98_000, SocialInsurancePrefecture::Okinawa, false),
        &params,
    )
    .unwrap();
    assert_eq!(result.health_related_amount.as_yen(), 4_626);
    assert_eq!(result.pension_amount.as_yen(), 8_967);
    assert_eq!(result.total_amount.as_yen(), 13_593);
}
