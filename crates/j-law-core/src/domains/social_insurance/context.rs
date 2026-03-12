use std::collections::HashSet;

use crate::types::date::LegalDate;

use super::policy::SocialInsurancePolicy;

/// 協会けんぽ都道府県支部コード。
///
/// # 法的根拠
/// 健康保険法 第160条
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocialInsurancePrefecture {
    Hokkaido = 1,
    Aomori = 2,
    Iwate = 3,
    Miyagi = 4,
    Akita = 5,
    Yamagata = 6,
    Fukushima = 7,
    Ibaraki = 8,
    Tochigi = 9,
    Gunma = 10,
    Saitama = 11,
    Chiba = 12,
    Tokyo = 13,
    Kanagawa = 14,
    Niigata = 15,
    Toyama = 16,
    Ishikawa = 17,
    Fukui = 18,
    Yamanashi = 19,
    Nagano = 20,
    Gifu = 21,
    Shizuoka = 22,
    Aichi = 23,
    Mie = 24,
    Shiga = 25,
    Kyoto = 26,
    Osaka = 27,
    Hyogo = 28,
    Nara = 29,
    Wakayama = 30,
    Tottori = 31,
    Shimane = 32,
    Okayama = 33,
    Hiroshima = 34,
    Yamaguchi = 35,
    Tokushima = 36,
    Kagawa = 37,
    Ehime = 38,
    Kochi = 39,
    Fukuoka = 40,
    Saga = 41,
    Nagasaki = 42,
    Kumamoto = 43,
    Oita = 44,
    Miyazaki = 45,
    Kagoshima = 46,
    Okinawa = 47,
}

impl SocialInsurancePrefecture {
    /// 支部コードから都道府県を復元する。
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Hokkaido),
            2 => Some(Self::Aomori),
            3 => Some(Self::Iwate),
            4 => Some(Self::Miyagi),
            5 => Some(Self::Akita),
            6 => Some(Self::Yamagata),
            7 => Some(Self::Fukushima),
            8 => Some(Self::Ibaraki),
            9 => Some(Self::Tochigi),
            10 => Some(Self::Gunma),
            11 => Some(Self::Saitama),
            12 => Some(Self::Chiba),
            13 => Some(Self::Tokyo),
            14 => Some(Self::Kanagawa),
            15 => Some(Self::Niigata),
            16 => Some(Self::Toyama),
            17 => Some(Self::Ishikawa),
            18 => Some(Self::Fukui),
            19 => Some(Self::Yamanashi),
            20 => Some(Self::Nagano),
            21 => Some(Self::Gifu),
            22 => Some(Self::Shizuoka),
            23 => Some(Self::Aichi),
            24 => Some(Self::Mie),
            25 => Some(Self::Shiga),
            26 => Some(Self::Kyoto),
            27 => Some(Self::Osaka),
            28 => Some(Self::Hyogo),
            29 => Some(Self::Nara),
            30 => Some(Self::Wakayama),
            31 => Some(Self::Tottori),
            32 => Some(Self::Shimane),
            33 => Some(Self::Okayama),
            34 => Some(Self::Hiroshima),
            35 => Some(Self::Yamaguchi),
            36 => Some(Self::Tokushima),
            37 => Some(Self::Kagawa),
            38 => Some(Self::Ehime),
            39 => Some(Self::Kochi),
            40 => Some(Self::Fukuoka),
            41 => Some(Self::Saga),
            42 => Some(Self::Nagasaki),
            43 => Some(Self::Kumamoto),
            44 => Some(Self::Oita),
            45 => Some(Self::Miyazaki),
            46 => Some(Self::Kagoshima),
            47 => Some(Self::Okinawa),
            _ => None,
        }
    }

    /// 支部コードを返す。
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// 社会保険料計算フラグ。
///
/// # 法的根拠
/// 介護保険法 第129条
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocialInsuranceFlag {
    /// 介護保険第2号被保険者として介護保険料を合算する。
    ///
    /// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
    /// 呼び出し元が正しく判断した上で指定してください。
    IsCareInsuranceApplicable,
}

/// 月額社会保険料計算のコンテキスト。
///
/// このドメインは、協会けんぽ一般被保険者について、
/// すでに決定済みの標準報酬月額を入力として本人負担分を計算する。
/// 算定基礎届・月額変更届による標準報酬月額の決定自体は対象外とする。
///
/// # 法的根拠
/// 健康保険法 第160条
/// 介護保険法 第129条
/// 厚生年金保険法 第81条
pub struct SocialInsuranceContext {
    /// 健康保険の標準報酬月額（円）。
    pub standard_monthly_remuneration: u64,
    /// 保険料率の適用基準日（月額保険料の対象月の初日を想定）。
    pub target_date: LegalDate,
    /// 協会けんぽの支部都道府県。
    pub prefecture: SocialInsurancePrefecture,
    /// 適用フラグ。
    pub flags: HashSet<SocialInsuranceFlag>,
    /// 本人負担端数処理ポリシー。
    pub policy: Box<dyn SocialInsurancePolicy>,
}
