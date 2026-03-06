use std::collections::HashSet;

use crate::domains::consumption_tax::policy::ConsumptionTaxPolicy;
use crate::types::date::LegalDate;

/// 消費税計算に影響するフラグ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumptionTaxFlag {
    /// 軽減税率（8%）を適用する。
    ///
    /// # WARNING
    /// 「軽減税率の対象品目に該当するか」の事実認定はライブラリの責任範囲外。
    /// 呼び出し元が適切に判断してこのフラグを設定すること。
    /// 対象品目: 飲食料品（酒類・外食を除く）、定期購読の新聞（2019年10月1日〜）。
    ReducedRate,
}

/// 消費税計算のコンテキスト。
#[derive(Debug)]
pub struct ConsumptionTaxContext {
    /// 課税標準額（税抜き金額・円）。
    pub amount: u64,
    /// 計算対象日。
    pub target_date: LegalDate,
    /// 適用フラグ。
    pub flags: HashSet<ConsumptionTaxFlag>,
    /// 端数処理・特例判定ポリシー。
    pub policy: Box<dyn ConsumptionTaxPolicy>,
}
