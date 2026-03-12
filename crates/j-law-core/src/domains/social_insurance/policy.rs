use std::collections::HashSet;

use super::context::SocialInsuranceFlag;

/// 被保険者負担分の端数処理モード。
///
/// # 法的根拠
/// 日本年金機構「保険料を給与から控除するとき、被保険者負担分の端数はどのように処理すればよいですか。」
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeShareRoundingMode {
    /// 給与・賞与から控除する場合。
    ///
    /// 50銭以下切り捨て、50銭超切り上げ。
    PayrollDeduction,
    /// 被保険者が現金で支払う場合。
    ///
    /// 50銭未満切り捨て、50銭以上切り上げ。
    CashPayment,
}

/// 社会保険料計算ポリシー。
pub trait SocialInsurancePolicy: std::fmt::Debug {
    /// 介護保険料を合算するか判定する。
    fn should_apply_care_insurance(&self, flags: &HashSet<SocialInsuranceFlag>) -> bool;

    /// 本人負担分の端数処理モードを返す。
    fn employee_share_rounding_mode(&self) -> EmployeeShareRoundingMode;
}

/// 日本年金機構の標準的な給与控除運用に合わせたポリシー。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
#[derive(Debug, Clone, Copy)]
pub struct StandardNenkinPolicy;

impl SocialInsurancePolicy for StandardNenkinPolicy {
    fn should_apply_care_insurance(&self, flags: &HashSet<SocialInsuranceFlag>) -> bool {
        flags.contains(&SocialInsuranceFlag::IsCareInsuranceApplicable)
    }

    fn employee_share_rounding_mode(&self) -> EmployeeShareRoundingMode {
        EmployeeShareRoundingMode::PayrollDeduction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_policy_applies_care_with_flag() {
        let mut flags = HashSet::new();
        flags.insert(SocialInsuranceFlag::IsCareInsuranceApplicable);
        assert!(StandardNenkinPolicy.should_apply_care_insurance(&flags));
    }

    #[test]
    fn standard_policy_defaults_to_payroll_deduction() {
        assert_eq!(
            StandardNenkinPolicy.employee_share_rounding_mode(),
            EmployeeShareRoundingMode::PayrollDeduction
        );
    }
}
