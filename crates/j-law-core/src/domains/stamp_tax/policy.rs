use crate::domains::stamp_tax::context::StampTaxContext;
use crate::domains::stamp_tax::params::{StampTaxDocumentParams, StampTaxSpecialRule};

/// 印紙税の計算ポリシー。
pub trait StampTaxPolicy: std::fmt::Debug {
    /// 適用すべき特例ルールを選択する。
    ///
    /// # 法的根拠
    /// 印紙税法 別表第一 / 租税特別措置法 第91条
    fn select_special_rule<'a>(
        &self,
        ctx: &StampTaxContext,
        document: &'a StampTaxDocumentParams,
    ) -> Option<&'a StampTaxSpecialRule>;
}

/// 国税庁の標準ポリシー。
#[derive(Debug, Clone, Copy)]
pub struct StandardNtaPolicy;

impl StampTaxPolicy for StandardNtaPolicy {
    fn select_special_rule<'a>(
        &self,
        ctx: &StampTaxContext,
        document: &'a StampTaxDocumentParams,
    ) -> Option<&'a StampTaxSpecialRule> {
        let date_str = ctx.target_date.to_date_str();
        document
            .special_rules
            .iter()
            .filter(|rule| rule.matches_date(&date_str))
            .filter(|rule| {
                rule.required_flags
                    .iter()
                    .all(|flag| ctx.flags.contains(flag))
            })
            .filter(|rule| rule.matches_amount(ctx.stated_amount))
            .min_by_key(|rule| rule.priority)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::domains::stamp_tax::context::{StampTaxDocumentCode, StampTaxFlag};
    use crate::domains::stamp_tax::params::{
        StampTaxAmountUsage, StampTaxBracket, StampTaxChargeMode, StampTaxCitation,
    };
    use crate::types::date::LegalDate;

    #[test]
    fn standard_policy_selects_matching_rule() {
        let document = StampTaxDocumentParams {
            code: StampTaxDocumentCode::Article1RealEstateTransfer,
            label: "第1号".into(),
            citation: StampTaxCitation {
                law_name: "印紙税法".into(),
                article: "別表第一 第1号文書".into(),
            },
            charge_mode: StampTaxChargeMode::AmountBrackets,
            amount_usage: StampTaxAmountUsage::Optional,
            base_rule_label: "通常".into(),
            base_tax_amount: None,
            brackets: vec![],
            no_amount_tax_amount: Some(200),
            no_amount_rule_label: Some("記載なし".into()),
            special_rules: vec![StampTaxSpecialRule {
                code: "reduced".into(),
                label: "軽減".into(),
                priority: 1,
                effective_from: Some("2014-04-01".into()),
                effective_until: Some("2027-03-31".into()),
                required_flags: vec![StampTaxFlag::Article17NonBusinessExempt],
                tax_amount: Some(0),
                rule_label: Some("非課税".into()),
                brackets: vec![StampTaxBracket {
                    label: "50万円以下".into(),
                    amount_from: 0,
                    amount_to_inclusive: Some(500_000),
                    tax_amount: 0,
                }],
                no_amount_tax_amount: None,
                no_amount_rule_label: None,
            }],
        };

        let mut flags = HashSet::new();
        flags.insert(StampTaxFlag::Article17NonBusinessExempt);
        let ctx = StampTaxContext {
            document_code: StampTaxDocumentCode::Article1RealEstateTransfer,
            stated_amount: Some(100_000),
            target_date: LegalDate::new(2024, 1, 1),
            flags,
            policy: Box::new(StandardNtaPolicy),
        };

        let rule = StandardNtaPolicy
            .select_special_rule(&ctx, &document)
            .unwrap();
        assert_eq!(rule.code, "reduced");
    }
}
