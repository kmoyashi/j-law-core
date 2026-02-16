use std::collections::HashSet;
use crate::domains::real_estate::policy::RealEstatePolicy;

/// 不動産取引計算に関わる法的フラグ。
///
/// WARNING: 各フラグの事実認定はライブラリの責任範囲外です。
/// 呼び出し元が正しく判断した上で指定してください。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealEstateFlag {
    /// 低廉な空き家特例を申請する場合に指定する。
    ///
    /// 適用要件: 宅地建物取引業法 第46条 / 国土交通省告示（2024年7月1日施行）
    /// WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
    IsLowCostVacantHouse,
    /// 商業物件フラグ。
    IsCommercialProperty,
    /// 双方代理（売主・買主双方から報酬を受領）フラグ。
    IsDualSide,
}

/// 媒介報酬計算の入力コンテキスト。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
pub struct RealEstateContext {
    /// 売買価格（円）。
    pub price: u64,
    /// 契約日・適用する告示を選択するための基準日 `(year, month, day)`。
    pub target_date: (u16, u8, u8),
    /// 適用する法的フラグの集合。
    pub flags: HashSet<RealEstateFlag>,
    /// 計算ポリシー（テスト・カスタム計算での差し替えを可能にする）。
    pub policy: Box<dyn RealEstatePolicy>,
}
