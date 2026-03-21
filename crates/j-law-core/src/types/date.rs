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

    /// ISO 8601 形式（"YYYY-MM-DD"）の文字列からパースする。
    ///
    /// 不正な形式の場合は `None` を返す。
    pub fn from_date_str(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        // "YYYY-MM-DD" = 10 バイト固定
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year: u16 = s[0..4].parse().ok()?;
        let month: u8 = s[5..7].parse().ok()?;
        let day: u8 = s[8..10].parse().ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > Self::days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// 当該年が閏年かどうかを返す。
    ///
    /// グレゴリオ暦の規則: 4で割り切れる && (100で割り切れない || 400で割り切れる)
    fn is_leap_year(year: u16) -> bool {
        let y = year as u32;
        y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400))
    }

    /// 指定した月の日数を返す。
    fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0, // 不正な月（呼び出し側で保証）
        }
    }

    /// 翌日の `LegalDate` を返す。
    ///
    /// 月末・年末の繰り上がりを正確に処理する（chrono 不使用、純粋算術ベース）。
    pub fn next_day(&self) -> Self {
        let max_day = Self::days_in_month(self.year, self.month);
        if self.day < max_day {
            // 月内で翌日
            Self::new(self.year, self.month, self.day + 1)
        } else if self.month < 12 {
            // 翌月1日
            Self::new(self.year, self.month + 1, 1)
        } else {
            // 翌年1月1日
            Self::new(self.year + 1, 1, 1)
        }
    }
}

impl From<(u16, u8, u8)> for LegalDate {
    /// タプル `(year, month, day)` から `LegalDate` を構築する。
    ///
    /// 既存コードとの互換性のために提供する。
    fn from((year, month, day): (u16, u8, u8)) -> Self {
        Self::new(year, month, day)
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

    #[test]
    fn from_date_str_valid() {
        let d = LegalDate::from_date_str("2024-07-01").unwrap();
        assert_eq!(d, LegalDate::new(2024, 7, 1));
    }

    #[test]
    fn from_date_str_invalid() {
        assert!(LegalDate::from_date_str("2024-7-1").is_none());
        assert!(LegalDate::from_date_str("20240701").is_none());
        assert!(LegalDate::from_date_str("not-a-date").is_none());
    }

    #[test]
    fn from_date_str_rejects_impossible_dates() {
        // 2月29日は平年では不正
        assert!(LegalDate::from_date_str("2023-02-29").is_none());
        // 2月29日は閏年では正当
        assert!(LegalDate::from_date_str("2024-02-29").is_some());
        // 4月31日は存在しない
        assert!(LegalDate::from_date_str("2024-04-31").is_none());
        // 6月31日は存在しない
        assert!(LegalDate::from_date_str("2024-06-31").is_none());
        // 月=13 は不正
        assert!(LegalDate::from_date_str("2024-13-01").is_none());
        // 日=0 は不正
        assert!(LegalDate::from_date_str("2024-01-00").is_none());
    }

    #[test]
    fn is_leap_year_cases() {
        // 400の倍数: 閏年
        assert!(LegalDate::is_leap_year(2000));
        // 100の倍数だが400の倍数でない: 平年
        assert!(!LegalDate::is_leap_year(1900));
        // 4の倍数で100の倍数でない: 閏年
        assert!(LegalDate::is_leap_year(2024));
        // 4の倍数でない: 平年
        assert!(!LegalDate::is_leap_year(2023));
    }

    #[test]
    fn next_day_normal() {
        // 月内の翌日
        assert_eq!(
            LegalDate::new(2024, 7, 15).next_day(),
            LegalDate::new(2024, 7, 16)
        );
    }

    #[test]
    fn next_day_month_end_30() {
        // 30日月末 → 翌月1日
        assert_eq!(
            LegalDate::new(2024, 6, 30).next_day(),
            LegalDate::new(2024, 7, 1)
        );
    }

    #[test]
    fn next_day_month_end_31() {
        // 31日月末 → 翌月1日
        assert_eq!(
            LegalDate::new(2024, 7, 31).next_day(),
            LegalDate::new(2024, 8, 1)
        );
    }

    #[test]
    fn next_day_year_end() {
        // 年末 → 翌年1月1日
        assert_eq!(
            LegalDate::new(2024, 12, 31).next_day(),
            LegalDate::new(2025, 1, 1)
        );
    }

    #[test]
    fn next_day_feb_28_non_leap() {
        // 平年の2月28日 → 3月1日
        assert_eq!(
            LegalDate::new(2023, 2, 28).next_day(),
            LegalDate::new(2023, 3, 1)
        );
    }

    #[test]
    fn next_day_feb_28_leap() {
        // 閏年の2月28日 → 2月29日
        assert_eq!(
            LegalDate::new(2024, 2, 28).next_day(),
            LegalDate::new(2024, 2, 29)
        );
    }

    #[test]
    fn next_day_feb_29_leap() {
        // 閏年の2月29日 → 3月1日
        assert_eq!(
            LegalDate::new(2024, 2, 29).next_day(),
            LegalDate::new(2024, 3, 1)
        );
    }
}
