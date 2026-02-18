use serde::Deserialize;

use crate::schema::Fraction;

/// 所得税の税率ブラケット（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxBracketEntry {
    pub label: String,
    pub income_from: u64,
    pub income_to_inclusive: Option<u64>,
    pub rate: Fraction,
    pub deduction: u64,
}

/// 復興特別所得税パラメータ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct ReconstructionTaxEntry {
    pub rate: Fraction,
    pub effective_from_year: u16,
    pub effective_to_year_inclusive: u16,
}

/// 所得税の計算パラメータ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxParamsEntry {
    pub brackets: Vec<IncomeTaxBracketEntry>,
    pub reconstruction_tax: Option<ReconstructionTaxEntry>,
}

/// 所得税の履歴エントリ（JSON）。
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxHistoryEntry {
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub params: IncomeTaxParamsEntry,
}

/// `income_tax.json` のルートスキーマ。
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxRegistry {
    pub domain: String,
    pub history: Vec<IncomeTaxHistoryEntry>,
}
