use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use crate::domains::withholding_tax::policy::WithholdingTaxPolicy;
use crate::error::InputError;
use crate::types::date::LegalDate;

/// 報酬・料金等の源泉徴収カテゴリ。
///
/// # 法的根拠
/// 所得税法 第204条第1項
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WithholdingTaxCategory {
    /// 原稿料・講演料等。
    ///
    /// # 法的根拠
    /// 所得税法 第204条第1項第1号
    ManuscriptAndLecture,
    /// 弁護士・税理士・公認会計士等の報酬・料金。
    ///
    /// # 法的根拠
    /// 所得税法 第204条第1項第2号
    ProfessionalFee,
    /// 役務の提供等を約することにより一時に支払う契約金。
    ///
    /// # 法的根拠
    /// 所得税法 第204条第1項第7号
    ExclusiveContractFee,
}

impl WithholdingTaxCategory {
    /// 永続化・バインディング用のコード文字列。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManuscriptAndLecture => "manuscript_and_lecture",
            Self::ProfessionalFee => "professional_fee",
            Self::ExclusiveContractFee => "exclusive_contract_fee",
        }
    }

    /// C ABI 用の整数コード。
    pub fn ffi_code(self) -> u32 {
        match self {
            Self::ManuscriptAndLecture => 1,
            Self::ProfessionalFee => 2,
            Self::ExclusiveContractFee => 3,
        }
    }

    /// C ABI から整数コードを復元する。
    pub fn from_ffi_code(code: u32) -> Result<Self, InputError> {
        match code {
            1 => Ok(Self::ManuscriptAndLecture),
            2 => Ok(Self::ProfessionalFee),
            3 => Ok(Self::ExclusiveContractFee),
            _ => Err(InputError::InvalidWithholdingInput {
                field: "category".into(),
                reason: format!("未知のカテゴリコードです: {code}"),
            }),
        }
    }
}

impl fmt::Display for WithholdingTaxCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<WithholdingTaxCategory> for u32 {
    fn from(value: WithholdingTaxCategory) -> Self {
        value.ffi_code()
    }
}

impl FromStr for WithholdingTaxCategory {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manuscript_and_lecture" => Ok(Self::ManuscriptAndLecture),
            "professional_fee" => Ok(Self::ProfessionalFee),
            "exclusive_contract_fee" => Ok(Self::ExclusiveContractFee),
            _ => Err(InputError::InvalidWithholdingInput {
                field: "category".into(),
                reason: format!("未知のカテゴリです: {s}"),
            }),
        }
    }
}

/// 源泉徴収税額の計算に影響するフラグ。
///
/// # 法的根拠
/// 所得税法 第204条第1項第1号
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WithholdingTaxFlag {
    /// 応募作品等の入選者に支払う賞金・謝金として扱う。
    ///
    /// 原稿料・講演料等のうち、1回の支払額が50,000円以下である場合は
    /// 源泉徴収不要となる。
    ///
    /// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    IsSubmissionPrize,
}

/// 報酬・料金等の源泉徴収税額計算コンテキスト。
///
/// # 法的根拠
/// 所得税法 第204条第1項
pub struct WithholdingTaxContext {
    /// 実際に支払う総額（円）。
    ///
    /// 請求書等で消費税額が区分表示されている場合でも、
    /// ここには支払総額を指定する。
    pub payment_amount: u64,
    /// 請求書等で区分表示された消費税額（円）。
    ///
    /// 0 の場合は区分表示なしとして扱う。
    pub separated_consumption_tax_amount: u64,
    /// 報酬・料金等のカテゴリ。
    pub category: WithholdingTaxCategory,
    /// 計算対象日。
    pub target_date: LegalDate,
    /// 適用フラグ。
    pub flags: HashSet<WithholdingTaxFlag>,
    /// 端数処理・特例判定ポリシー。
    pub policy: Box<dyn WithholdingTaxPolicy>,
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domains::withholding_tax::policy::StandardWithholdingTaxPolicy;

    #[test]
    fn category_string_roundtrip() {
        let category = WithholdingTaxCategory::from_str("professional_fee").unwrap();
        assert_eq!(category, WithholdingTaxCategory::ProfessionalFee);
        assert_eq!(category.as_str(), "professional_fee");
    }

    #[test]
    fn category_ffi_roundtrip() {
        let category = WithholdingTaxCategory::from_ffi_code(3).unwrap();
        assert_eq!(category, WithholdingTaxCategory::ExclusiveContractFee);
        assert_eq!(category.ffi_code(), 3);
    }

    #[test]
    fn context_construction() {
        let ctx = WithholdingTaxContext {
            payment_amount: 100_000,
            separated_consumption_tax_amount: 10_000,
            category: WithholdingTaxCategory::ManuscriptAndLecture,
            target_date: LegalDate::new(2026, 1, 1),
            flags: HashSet::new(),
            policy: Box::new(StandardWithholdingTaxPolicy),
        };

        assert_eq!(ctx.payment_amount, 100_000);
        assert_eq!(ctx.separated_consumption_tax_amount, 10_000);
    }
}
