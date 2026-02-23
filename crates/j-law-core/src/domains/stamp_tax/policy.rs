use std::collections::HashSet;

use super::context::StampTaxFlag;

/// 印紙税の計算ポリシー。
///
/// 軽減措置の適用判定ロジックを抽象化する。
pub trait StampTaxPolicy: std::fmt::Debug {
    /// 軽減税率を適用すべきか判定する。
    ///
    /// # 法的根拠
    /// 租税特別措置法 第91条（軽減措置の適用要件）
    ///
    /// # 引数
    /// - `date_str`: 対象日（ISO 8601形式、例: "2024-08-01"）
    /// - `reduced_from`: 軽減措置の適用開始日
    /// - `reduced_until`: 軽減措置の適用終了日
    /// - `flags`: 適用フラグ
    fn should_apply_reduced_rate(
        &self,
        date_str: &str,
        reduced_from: Option<&str>,
        reduced_until: Option<&str>,
        flags: &HashSet<StampTaxFlag>,
    ) -> bool;
}

/// 国税庁の標準ポリシー。
///
/// 軽減措置の適用条件:
/// 1. `IsReducedTaxRateApplicable` フラグが指定されていること
/// 2. 対象日が軽減措置の適用期間内であること
#[derive(Debug, Clone, Copy)]
pub struct StandardNtaPolicy;

impl StampTaxPolicy for StandardNtaPolicy {
    fn should_apply_reduced_rate(
        &self,
        date_str: &str,
        reduced_from: Option<&str>,
        reduced_until: Option<&str>,
        flags: &HashSet<StampTaxFlag>,
    ) -> bool {
        if !flags.contains(&StampTaxFlag::IsReducedTaxRateApplicable) {
            return false;
        }
        match (reduced_from, reduced_until) {
            (Some(from), Some(until)) => date_str >= from && date_str <= until,
            (Some(from), None) => date_str >= from,
            (None, Some(until)) => date_str <= until,
            (None, None) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags_with_reduced() -> HashSet<StampTaxFlag> {
        let mut flags = HashSet::new();
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
        flags
    }

    #[test]
    fn standard_policy_applies_within_period() {
        let policy = StandardNtaPolicy;
        assert!(policy.should_apply_reduced_rate(
            "2024-08-01",
            Some("2014-04-01"),
            Some("2027-03-31"),
            &flags_with_reduced(),
        ));
    }

    #[test]
    fn standard_policy_rejects_before_period() {
        let policy = StandardNtaPolicy;
        assert!(!policy.should_apply_reduced_rate(
            "2014-03-31",
            Some("2014-04-01"),
            Some("2027-03-31"),
            &flags_with_reduced(),
        ));
    }

    #[test]
    fn standard_policy_rejects_after_period() {
        let policy = StandardNtaPolicy;
        assert!(!policy.should_apply_reduced_rate(
            "2027-04-01",
            Some("2014-04-01"),
            Some("2027-03-31"),
            &flags_with_reduced(),
        ));
    }

    #[test]
    fn standard_policy_rejects_without_flag() {
        let policy = StandardNtaPolicy;
        assert!(!policy.should_apply_reduced_rate(
            "2024-08-01",
            Some("2014-04-01"),
            Some("2027-03-31"),
            &HashSet::new(),
        ));
    }
}
