use thiserror::Error;

/// Registry（告示JSONデータ）の不整合エラー。
/// 起動時バリデーションで検出し、panic! で伝播させてよい層。
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("法令データに適用期間の重複があります: domain={domain}, from={from}, until={until}")]
    PeriodOverlap {
        domain: String,
        from: String,
        until: String,
    },

    #[error(
        "法令データに適用期間の空白があります: domain={domain}, end={end}, next_start={next_start}"
    )]
    PeriodGap {
        domain: String,
        end: String,
        next_start: String,
    },

    #[error("法令データに浮動小数点値が含まれています（整数または分数形式を使用してください）: path={path}")]
    FloatProhibited { path: String },

    #[error("分母にゼロが含まれています: path={path}")]
    ZeroDenominator { path: String },

    #[error("JSONファイルが見つかりません: {path}")]
    FileNotFound { path: String },

    #[error("JSONのパースに失敗しました: path={path}, cause={cause}")]
    ParseError { path: String, cause: String },
}

/// ユーザー入力の不正エラー。
#[derive(Debug, Error)]
pub enum InputError {
    #[error("負の金額は無効です: value={value}")]
    NegativeAmount { value: i64 },

    #[error("指定した日付は法令の適用期間外です: date={date}")]
    DateOutOfRange { date: String },

    #[error("矛盾するフラグが同時に指定されています: {flag_a} と {flag_b}")]
    ConflictingFlags { flag_a: String, flag_b: String },

    #[error("所得控除の入力が無効です: field={field}, reason={reason}")]
    InvalidDeductionInput { field: String, reason: String },

    #[error("源泉徴収入力が無効です: field={field}, reason={reason}")]
    InvalidWithholdingInput { field: String, reason: String },

    #[error("分母にゼロが指定されました")]
    ZeroDenominator,
}

/// 計算処理中の異常エラー。
#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("計算中に整数オーバーフローが発生しました: step={step}")]
    Overflow { step: String },

    #[error("このコンテキストにポリシーを適用できません: {reason}")]
    PolicyNotApplicable { reason: String },
}

/// J-Law-Core 全体のトップレベルエラー型。
#[derive(Debug, Error)]
pub enum JLawError {
    #[error(transparent)]
    Registry(#[from] RegistryError),

    #[error(transparent)]
    Input(#[from] InputError),

    #[error(transparent)]
    Calculation(#[from] CalculationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_error_display() {
        let e = RegistryError::PeriodOverlap {
            domain: "real_estate".into(),
            from: "2024-01-01".into(),
            until: "2024-12-31".into(),
        };
        assert!(e.to_string().contains("real_estate"));
        assert!(e.to_string().contains("重複"));
    }

    #[test]
    fn input_error_display() {
        let e = InputError::NegativeAmount { value: -100 };
        assert!(e.to_string().contains("-100"));
    }

    #[test]
    fn invalid_deduction_input_display() {
        let e = InputError::InvalidDeductionInput {
            field: "social_insurance_premium_paid".into(),
            reason: "test".into(),
        };
        assert!(e.to_string().contains("social_insurance_premium_paid"));
        assert!(e.to_string().contains("test"));
    }

    #[test]
    fn invalid_withholding_input_display() {
        let e = InputError::InvalidWithholdingInput {
            field: "category".into(),
            reason: "unknown".into(),
        };
        assert!(e.to_string().contains("category"));
        assert!(e.to_string().contains("unknown"));
    }

    #[test]
    fn calculation_error_display() {
        let e = CalculationError::Overflow {
            step: "tier1".into(),
        };
        assert!(e.to_string().contains("tier1"));
    }

    #[test]
    fn jlaw_error_from_input() {
        let inner = InputError::ZeroDenominator;
        let outer: JLawError = inner.into();
        assert!(matches!(outer, JLawError::Input(_)));
    }

    #[test]
    fn jlaw_error_from_calculation() {
        let inner = CalculationError::PolicyNotApplicable {
            reason: "test".into(),
        };
        let outer: JLawError = inner.into();
        assert!(matches!(outer, JLawError::Calculation(_)));
    }
}
