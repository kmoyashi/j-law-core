use crate::types::date::LegalDate;

/// 法令の条文参照情報。
///
/// `pub` な型・関数の docコメントに埋め込むことで、
/// 実装の根拠を機械可読な形で記録する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalCitation {
    /// 法令識別子（e.g., `"reitaku-46"`, `"shotoku-tax-36"`）。
    pub law_id: String,
    /// 法令の正式名称。
    pub law_name: String,
    /// 条番号。
    pub article: u16,
    /// 項番号（省略可）。
    pub paragraph: Option<u16>,
    /// 号番号（省略可）。
    pub item: Option<u16>,
    /// 施行日。
    pub effective_date: LegalDate,
}

impl LegalCitation {
    /// 簡易コンストラクタ（条のみ指定）。
    pub fn article_only(
        law_id: &str,
        law_name: &str,
        article: u16,
        effective_date: LegalDate,
    ) -> Self {
        Self {
            law_id: law_id.to_owned(),
            law_name: law_name.to_owned(),
            article,
            paragraph: None,
            item: None,
            effective_date,
        }
    }
}

impl std::fmt::Display for LegalCitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} 第{}条", self.law_name, self.article)?;
        if let Some(p) = self.paragraph {
            write!(f, "第{}項", p)?;
        }
        if let Some(i) = self.item {
            write!(f, "第{}号", i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_article_only() {
        let c = LegalCitation::article_only(
            "reitaku-46",
            "宅地建物取引業法",
            46,
            LegalDate::new(2024, 7, 1),
        );
        assert_eq!(c.to_string(), "宅地建物取引業法 第46条");
    }

    #[test]
    fn display_with_paragraph_and_item() {
        let c = LegalCitation {
            law_id: "reitaku-46".into(),
            law_name: "宅地建物取引業法".into(),
            article: 46,
            paragraph: Some(1),
            item: Some(2),
            effective_date: LegalDate::new(2024, 7, 1),
        };
        assert_eq!(c.to_string(), "宅地建物取引業法 第46条第1項第2号");
    }
}
