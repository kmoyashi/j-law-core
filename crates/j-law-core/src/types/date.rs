/// 法令の施行日・基準日を表す日付型。
///
/// 年月日の3要素で特定される暦日（西暦）。
/// 匿名タプル `(u16, u8, u8)` に代わる名前付き型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegalDate {
    /// 年（西暦）
    pub year: u16,
    /// 月（1〜12）
    pub month: u8,
    /// 日（1〜31）
    pub day: u8,
}

impl LegalDate {
    /// 年・月・日からインスタンスを作成する。
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// ISO 8601 形式（"YYYY-MM-DD"）の文字列に変換する。
    ///
    /// Registry JSON の日付文字列との比較に使用する。
    pub fn to_date_str(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_fields() {
        let d = LegalDate::new(2024, 8, 1);
        assert_eq!(d.year, 2024);
        assert_eq!(d.month, 8);
        assert_eq!(d.day, 1);
    }

    #[test]
    fn to_date_str_format() {
        assert_eq!(LegalDate::new(2024, 8, 1).to_date_str(), "2024-08-01");
        assert_eq!(LegalDate::new(2015, 1, 1).to_date_str(), "2015-01-01");
        assert_eq!(LegalDate::new(2024, 12, 31).to_date_str(), "2024-12-31");
    }
}
