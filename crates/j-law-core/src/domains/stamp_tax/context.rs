use std::collections::HashSet;

use crate::types::date::LegalDate;

use super::policy::StampTaxPolicy;

/// 印紙税の文書種別。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書 / 第2号文書
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StampTaxDocumentKind {
    /// 不動産の譲渡に関する契約書。
    ///
    /// 土地建物売買契約書など、印紙税法 別表第一 第1号文書のうち
    /// 租税特別措置法第91条の軽減措置対象となる文書。
    RealEstateTransfer,
    /// 建設工事の請負に関する契約書。
    ///
    /// 建物建築工事請負契約書など、印紙税法 別表第一 第2号文書のうち
    /// 租税特別措置法第91条の軽減措置対象となる文書。
    ConstructionContract,
}

/// 印紙税の適用フラグ。
///
/// # 法的根拠
/// 租税特別措置法 第91条
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
    /// 契約書の文書種別。
    pub document_kind: StampTaxDocumentKind,
    /// 契約金額（円）。
    pub contract_amount: u64,
    /// 契約書の作成日。
    pub target_date: LegalDate,
    /// 適用フラグ。
    pub flags: HashSet<StampTaxFlag>,
    /// 計算ポリシー。
    pub policy: Box<dyn StampTaxPolicy>,
}
