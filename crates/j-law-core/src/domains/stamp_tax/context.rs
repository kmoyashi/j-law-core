use std::collections::HashSet;

use super::policy::StampTaxPolicy;

/// 印紙税の適用フラグ。
///
/// # 法的根拠
/// 租税特別措置法 第91条（不動産の譲渡に関する契約書の印紙税の軽減措置）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StampTaxFlag {
    /// 軽減税率の適用対象であるか。
    ///
    /// 不動産の譲渡に関する契約書（第1号文書）または建設工事の請負に関する契約書（第2号文書）
    /// のうち、租税特別措置法に定める軽減措置の対象となるもの。
    ///
    /// WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任。
    IsReducedTaxRateApplicable,
}

/// 印紙税計算のコンテキスト。
///
/// # 法的根拠
/// 印紙税法 第2条（課税文書）/ 別表第一（課税物件表）
pub struct StampTaxContext {
    /// 契約金額（円）。
    pub contract_amount: u64,
    /// 契約書の作成日 (year, month, day)。
    pub target_date: (u16, u8, u8),
    /// 適用フラグ。
    pub flags: HashSet<StampTaxFlag>,
    /// 計算ポリシー。
    pub policy: Box<dyn StampTaxPolicy>,
}
