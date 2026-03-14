use std::collections::BTreeMap;
use std::str::FromStr;

use crate::domains::stamp_tax::context::{StampTaxDocumentCode, StampTaxFlag};
use crate::error::InputError;

/// 税額適用の法的根拠。
#[derive(Debug, Clone)]
pub struct StampTaxCitation {
    pub law_name: String,
    pub article: String,
}

/// 印紙税額のブラケット（1区間分）。
///
/// # 法的根拠
/// 印紙税法 別表第一
#[derive(Debug, Clone)]
pub struct StampTaxBracket {
    pub label: String,
    pub amount_from: u64,
    pub amount_to_inclusive: Option<u64>,
    pub tax_amount: u64,
}

impl StampTaxBracket {
    pub fn matches(&self, amount: u64) -> bool {
        amount >= self.amount_from
            && match self.amount_to_inclusive {
                Some(to) => amount <= to,
                None => true,
            }
    }
}

/// 税額表の課税モード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampTaxChargeMode {
    AmountBrackets,
    FixedPerDocument,
    FixedPerYear,
}

impl FromStr for StampTaxChargeMode {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "amount_brackets" => Ok(Self::AmountBrackets),
            "fixed_per_document" => Ok(Self::FixedPerDocument),
            "fixed_per_year" => Ok(Self::FixedPerYear),
            _ => Err(InputError::InvalidStampTaxInput {
                field: "charge_mode".into(),
                reason: format!("未知の charge_mode です: {s}"),
            }),
        }
    }
}

/// 文書コードごとの金額入力の扱い。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampTaxAmountUsage {
    Required,
    Optional,
    Unsupported,
}

impl FromStr for StampTaxAmountUsage {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "required" => Ok(Self::Required),
            "optional" => Ok(Self::Optional),
            "unsupported" => Ok(Self::Unsupported),
            _ => Err(InputError::InvalidStampTaxInput {
                field: "amount_usage".into(),
                reason: format!("未知の amount_usage です: {s}"),
            }),
        }
    }
}

/// 軽減・非課税などの特例ルール。
#[derive(Debug, Clone)]
pub struct StampTaxSpecialRule {
    pub code: String,
    pub label: String,
    pub priority: u16,
    pub effective_from: Option<String>,
    pub effective_until: Option<String>,
    pub required_flags: Vec<StampTaxFlag>,
    pub tax_amount: Option<u64>,
    pub rule_label: Option<String>,
    pub brackets: Vec<StampTaxBracket>,
    pub no_amount_tax_amount: Option<u64>,
    pub no_amount_rule_label: Option<String>,
}

impl StampTaxSpecialRule {
    pub fn matches_date(&self, date_str: &str) -> bool {
        let from_ok = match self.effective_from.as_deref() {
            Some(from) => date_str >= from,
            None => true,
        };
        let until_ok = match self.effective_until.as_deref() {
            Some(until) => date_str <= until,
            None => true,
        };
        from_ok && until_ok
    }

    pub fn matches_amount(&self, stated_amount: Option<u64>) -> bool {
        if self.tax_amount.is_some() {
            return true;
        }

        match stated_amount {
            Some(amount) => self.brackets.iter().any(|bracket| bracket.matches(amount)),
            None => self.no_amount_tax_amount.is_some(),
        }
    }
}

/// 文書コードごとのパラメータ。
///
/// # 法的根拠
/// 印紙税法 別表第一
#[derive(Debug, Clone)]
pub struct StampTaxDocumentParams {
    pub code: StampTaxDocumentCode,
    pub label: String,
    pub citation: StampTaxCitation,
    pub charge_mode: StampTaxChargeMode,
    pub amount_usage: StampTaxAmountUsage,
    pub base_rule_label: String,
    pub base_tax_amount: Option<u64>,
    pub brackets: Vec<StampTaxBracket>,
    pub no_amount_tax_amount: Option<u64>,
    pub no_amount_rule_label: Option<String>,
    pub special_rules: Vec<StampTaxSpecialRule>,
}

/// 印紙税計算に使うパラメータセット。
#[derive(Debug, Clone)]
pub struct StampTaxParams {
    pub documents: BTreeMap<StampTaxDocumentCode, StampTaxDocumentParams>,
}

impl StampTaxParams {
    pub(crate) fn document_params(
        &self,
        document_code: StampTaxDocumentCode,
    ) -> Option<&StampTaxDocumentParams> {
        self.documents.get(&document_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_mode_parses() {
        assert_eq!(
            StampTaxChargeMode::from_str("fixed_per_year").unwrap(),
            StampTaxChargeMode::FixedPerYear
        );
    }

    #[test]
    fn amount_usage_parses() {
        assert_eq!(
            StampTaxAmountUsage::from_str("optional").unwrap(),
            StampTaxAmountUsage::Optional
        );
    }

    #[test]
    fn bracket_match_inclusive_upper_bound() {
        let bracket = StampTaxBracket {
            label: "100万円以下".into(),
            amount_from: 100_000,
            amount_to_inclusive: Some(1_000_000),
            tax_amount: 200,
        };
        assert!(bracket.matches(1_000_000));
        assert!(!bracket.matches(1_000_001));
    }
}
