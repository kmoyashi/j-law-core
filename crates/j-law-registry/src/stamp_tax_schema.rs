use std::collections::BTreeMap;

use serde::Deserialize;

/// 印紙税法令根拠（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxCitationEntry {
    pub law_name: String,
    pub article: String,
}

/// 印紙税額のブラケット（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxBracketEntry {
    pub label: String,
    pub amount_from: u64,
    pub amount_to_inclusive: Option<u64>,
    pub tax_amount: u64,
}

/// 印紙税の特例ルール（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxSpecialRuleEntry {
    pub code: String,
    pub label: String,
    pub priority: u16,
    pub effective_from: Option<String>,
    pub effective_until: Option<String>,
    pub required_flags: Vec<String>,
    pub tax_amount: Option<u64>,
    pub rule_label: Option<String>,
    pub brackets: Vec<StampTaxBracketEntry>,
    pub no_amount_tax_amount: Option<u64>,
    pub no_amount_rule_label: Option<String>,
}

/// 印紙税の文書コードごとのパラメータ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxDocumentParamsEntry {
    pub label: String,
    pub citation: StampTaxCitationEntry,
    pub charge_mode: String,
    pub amount_usage: String,
    pub base_rule_label: String,
    pub base_tax_amount: Option<u64>,
    pub brackets: Vec<StampTaxBracketEntry>,
    pub no_amount_tax_amount: Option<u64>,
    pub no_amount_rule_label: Option<String>,
    pub special_rules: Vec<StampTaxSpecialRuleEntry>,
}

/// 印紙税の計算パラメータ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxParamsEntry {
    pub documents: BTreeMap<String, StampTaxDocumentParamsEntry>,
}

/// 印紙税の履歴エントリ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub params: StampTaxParamsEntry,
}

/// `stamp_tax.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct StampTaxRegistry {
    #[allow(dead_code)]
    pub domain: String,
    pub history: Vec<StampTaxHistoryEntry>,
}
