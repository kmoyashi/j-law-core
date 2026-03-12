use std::collections::HashSet;
use std::os::raw::{c_char, c_int};

use j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use j_law_core::domains::income_tax::{
    assessment::{calculate_income_tax_assessment, IncomeTaxAssessmentContext},
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
    deduction::calculate_income_deductions,
    deduction::{
        DependentDeductionInput, DonationDeductionInput, ExpenseDeductionInput,
        IncomeDeductionContext, IncomeDeductionInput, IncomeDeductionKind,
        LifeInsuranceDeductionInput, MedicalDeductionInput, PersonalDeductionInput,
        SpouseDeductionInput,
    },
    policy::StandardIncomeTaxPolicy,
};
use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee, context::RealEstateContext, policy::StandardMliitPolicy,
    RealEstateFlag,
};
use j_law_core::domains::stamp_tax::{
    calculator::calculate_stamp_tax,
    context::{StampTaxContext, StampTaxFlag},
    policy::StandardNtaPolicy,
};
use j_law_core::domains::withholding_tax::{
    calculator::calculate_withholding_tax,
    context::{WithholdingTaxCategory, WithholdingTaxContext, WithholdingTaxFlag},
    policy::StandardWithholdingTaxPolicy,
};
use j_law_core::{JLawError, LegalDate};
use j_law_registry::load_brokerage_fee_params;
use j_law_registry::load_consumption_tax_params;
use j_law_registry::load_income_tax_deduction_params;
use j_law_registry::load_income_tax_params;
use j_law_registry::load_stamp_tax_params;
use j_law_registry::load_withholding_tax_params;

// ─── 定数 ─────────────────────────────────────────────────────────────────────

/// ティア内訳の最大件数。現行法令では 3 ティアだが余裕を持たせる。
pub const J_LAW_MAX_TIERS: usize = 8;

/// 所得控除内訳の最大件数。
pub const J_LAW_MAX_DEDUCTION_LINES: usize = 8;

/// ティアラベルの最大バイト長（NUL 終端含む）。
pub const J_LAW_LABEL_LEN: usize = 64;

/// エラーバッファのデフォルトバイト長。Go 側のアロケーション目安。
pub const J_LAW_ERROR_BUF_LEN: usize = 256;

/// C FFI の互換バージョン。
pub const J_LAW_C_FFI_VERSION: u32 = 3;

/// 源泉徴収カテゴリ: 原稿料・講演料等。
pub const J_LAW_WITHHOLDING_TAX_CATEGORY_MANUSCRIPT_AND_LECTURE: u32 = 1;
/// 源泉徴収カテゴリ: 税理士等の報酬・料金。
pub const J_LAW_WITHHOLDING_TAX_CATEGORY_PROFESSIONAL_FEE: u32 = 2;
/// 源泉徴収カテゴリ: 専属契約金。
pub const J_LAW_WITHHOLDING_TAX_CATEGORY_EXCLUSIVE_CONTRACT_FEE: u32 = 3;

// ─── C 互換型定義 ──────────────────────────────────────────────────────────────

/// 1 ティアの計算内訳（C 互換）。
#[repr(C)]
pub struct JLawBreakdownStep {
    /// ティアラベル（NUL 終端・最大 63 文字）。
    pub label: [c_char; J_LAW_LABEL_LEN],
    /// ティア対象金額（円）。
    pub base_amount: u64,
    pub rate_numer: u64,
    pub rate_denom: u64,
    /// ティア計算結果（円・端数切捨て済み）。
    pub result: u64,
}

/// 媒介報酬の計算結果（C 互換）。
#[repr(C)]
pub struct JLawBrokerageFeeResult {
    /// 税抜合計額（円）。
    pub total_without_tax: u64,
    /// 税込合計額（円）。
    pub total_with_tax: u64,
    /// 消費税額（円）。
    pub tax_amount: u64,
    /// 低廉な空き家特例が適用されたか（0 = false, 1 = true）。
    pub low_cost_special_applied: c_int,
    /// 各ティアの計算内訳。
    pub breakdown: [JLawBreakdownStep; J_LAW_MAX_TIERS],
    /// breakdown の有効件数。
    pub breakdown_len: c_int,
}

// ─── 内部ユーティリティ ────────────────────────────────────────────────────────

/// UTF-8 文字列を固定長 `c_char` 配列に NUL 終端付きでコピーする。
///
/// `buf.len() - 1` バイトを超える場合はその位置で切り詰める。
fn copy_str_to_fixed_buf(s: &str, buf: &mut [c_char; J_LAW_LABEL_LEN]) {
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(J_LAW_LABEL_LEN - 1);
    for (i, &b) in bytes[..copy_len].iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf[copy_len] = 0;
}

/// エラーメッセージを呼び出し元バッファに書き込む。
///
/// # Safety
/// `buf` は `buf_len` バイト以上の有効なメモリ領域を指していること。
unsafe fn write_error_msg(msg: &str, buf: *mut c_char, buf_len: c_int) {
    let bytes = msg.as_bytes();
    let copy_len = bytes.len().min((buf_len - 1) as usize);
    for (i, &b) in bytes[..copy_len].iter().enumerate() {
        *buf.add(i) = b as c_char;
    }
    *buf.add(copy_len) = 0;
}

fn income_deduction_kind_to_c(kind: IncomeDeductionKind) -> u32 {
    match kind {
        IncomeDeductionKind::Basic => 1,
        IncomeDeductionKind::Spouse => 2,
        IncomeDeductionKind::Dependent => 3,
        IncomeDeductionKind::SocialInsurance => 4,
        IncomeDeductionKind::Medical => 5,
        IncomeDeductionKind::LifeInsurance => 6,
        IncomeDeductionKind::Donation => 7,
    }
}

fn try_count_to_u16(value: u64, field: &str) -> Result<u16, JLawError> {
    u16::try_from(value).map_err(|_| {
        JLawError::Input(j_law_core::InputError::InvalidDeductionInput {
            field: field.into(),
            reason: "u16 の上限を超えています".into(),
        })
    })
}

fn to_income_deduction_context(
    input: &JLawIncomeDeductionInput,
) -> Result<IncomeDeductionContext, JLawError> {
    let spouse = if input.has_spouse != 0 {
        Some(SpouseDeductionInput {
            spouse_total_income_amount: input.spouse_total_income_amount,
            is_same_household: input.spouse_is_same_household != 0,
            is_elderly: input.spouse_is_elderly != 0,
        })
    } else {
        None
    };
    let medical = if input.has_medical != 0 {
        Some(MedicalDeductionInput {
            medical_expense_paid: input.medical_expense_paid,
            reimbursed_amount: input.medical_reimbursed_amount,
        })
    } else {
        None
    };
    let life_insurance = if input.has_life_insurance != 0 {
        Some(LifeInsuranceDeductionInput {
            new_general_paid_amount: input.life_new_general_paid_amount,
            new_individual_pension_paid_amount: input.life_new_individual_pension_paid_amount,
            new_care_medical_paid_amount: input.life_new_care_medical_paid_amount,
            old_general_paid_amount: input.life_old_general_paid_amount,
            old_individual_pension_paid_amount: input.life_old_individual_pension_paid_amount,
        })
    } else {
        None
    };
    let donation = if input.has_donation != 0 {
        Some(DonationDeductionInput {
            qualified_donation_amount: input.donation_qualified_amount,
        })
    } else {
        None
    };

    Ok(IncomeDeductionContext {
        total_income_amount: input.total_income_amount,
        target_date: LegalDate::new(input.year, input.month, input.day),
        deductions: IncomeDeductionInput {
            personal: PersonalDeductionInput {
                spouse,
                dependent: DependentDeductionInput {
                    general_count: try_count_to_u16(
                        input.dependent_general_count,
                        "dependent.general_count",
                    )?,
                    specific_count: try_count_to_u16(
                        input.dependent_specific_count,
                        "dependent.specific_count",
                    )?,
                    elderly_cohabiting_count: try_count_to_u16(
                        input.dependent_elderly_cohabiting_count,
                        "dependent.elderly_cohabiting_count",
                    )?,
                    elderly_other_count: try_count_to_u16(
                        input.dependent_elderly_other_count,
                        "dependent.elderly_other_count",
                    )?,
                },
            },
            expense: ExpenseDeductionInput {
                social_insurance_premium_paid: input.social_insurance_premium_paid,
                medical,
                life_insurance,
                donation,
            },
        },
    })
}

fn write_income_tax_breakdown(
    breakdown: &[j_law_core::domains::income_tax::IncomeTaxStep],
    out_breakdown: &mut [JLawIncomeTaxStep; J_LAW_MAX_TIERS],
) -> c_int {
    let len = breakdown.len().min(J_LAW_MAX_TIERS);
    for (i, step) in breakdown.iter().take(J_LAW_MAX_TIERS).enumerate() {
        out_breakdown[i].taxable_income = step.taxable_income;
        out_breakdown[i].rate_numer = step.rate_numer;
        out_breakdown[i].rate_denom = step.rate_denom;
        out_breakdown[i].deduction = step.deduction;
        out_breakdown[i].result = step.result.as_yen();
        copy_str_to_fixed_buf(&step.label, &mut out_breakdown[i].label);
    }
    len as c_int
}

fn write_income_deduction_breakdown(
    breakdown: &[j_law_core::domains::income_tax::IncomeDeductionLine],
    out_breakdown: &mut [JLawIncomeDeductionLine; J_LAW_MAX_DEDUCTION_LINES],
) -> c_int {
    let len = breakdown.len().min(J_LAW_MAX_DEDUCTION_LINES);
    for (i, line) in breakdown.iter().take(J_LAW_MAX_DEDUCTION_LINES).enumerate() {
        out_breakdown[i].kind = income_deduction_kind_to_c(line.kind);
        out_breakdown[i].amount = line.amount.as_yen();
        copy_str_to_fixed_buf(&line.label, &mut out_breakdown[i].label);
    }
    len as c_int
}

fn write_breakdown(
    breakdown: &[j_law_core::domains::withholding_tax::WithholdingTaxStep],
    out_breakdown: &mut [JLawBreakdownStep; J_LAW_MAX_TIERS],
) -> c_int {
    let len = breakdown.len().min(J_LAW_MAX_TIERS);
    for (i, step) in breakdown.iter().take(J_LAW_MAX_TIERS).enumerate() {
        out_breakdown[i].base_amount = step.base_amount;
        out_breakdown[i].rate_numer = step.rate_numer;
        out_breakdown[i].rate_denom = step.rate_denom;
        out_breakdown[i].result = step.result.as_yen();
        copy_str_to_fixed_buf(&step.label, &mut out_breakdown[i].label);
    }
    len as c_int
}

// ─── C FFI 公開関数 ────────────────────────────────────────────────────────────

/// j-law-c-ffi の FFI バージョンを返す。
///
/// # 法的根拠
/// なし（FFI 互換確認用）
#[no_mangle]
pub extern "C" fn j_law_c_ffi_version() -> u32 {
    J_LAW_C_FFI_VERSION
}

/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項 / 国土交通省告示
///
/// # 引数
/// - `price`: 売買価格（円）
/// - `year`, `month`, `day`: 基準日
/// - `is_low_cost_vacant_house`: 低廉な空き家特例フラグ（0 = false, 非0 = true）
///   WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
/// - `is_seller`: 売主側フラグ（0 = false, 非0 = true）
///   2018年1月1日〜2024年6月30日の低廉特例は売主のみに適用される。
///   WARNING: 売主・買主の事実認定は呼び出し元の責任。
/// - `out_result`: [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
/// - `error_buf`: [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
/// - `error_buf_len`: `error_buf` のバイト長（推奨: `J_LAW_ERROR_BUF_LEN` = 256）
///
/// # 戻り値
/// - `0`: 成功。`out_result` にデータが書き込まれている。
/// - `非0`: 失敗。`error_buf` に NUL 終端エラーメッセージが書き込まれている。
///
/// # Safety
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
/// - `error_buf_len` は 1 以上であること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_brokerage_fee(
    price: u64,
    year: u16,
    month: u8,
    day: u8,
    is_low_cost_vacant_house: c_int,
    is_seller: c_int,
    out_result: *mut JLawBrokerageFeeResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    // パラメータロード
    let params = match load_brokerage_fee_params(LegalDate::new(year, month, day)) {
        Ok(p) => p,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // フラグ構築
    let mut flags = HashSet::new();
    if is_low_cost_vacant_house != 0 {
        flags.insert(RealEstateFlag::IsLowCostVacantHouse);
    }
    if is_seller != 0 {
        flags.insert(RealEstateFlag::IsSeller);
    }

    let ctx = RealEstateContext {
        price,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardMliitPolicy),
    };

    // 計算実行
    let result = match calculate_brokerage_fee(&ctx, &params) {
        Ok(r) => r,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // 結果を out_result に書き込む
    let out = &mut *out_result;
    out.total_without_tax = result.total_without_tax.as_yen();
    out.total_with_tax = result.total_with_tax.as_yen();
    out.tax_amount = result.tax_amount.as_yen();
    out.low_cost_special_applied = if result.low_cost_special_applied {
        1
    } else {
        0
    };
    out.breakdown_len = result.breakdown.len().min(J_LAW_MAX_TIERS) as c_int;

    for (i, step) in result.breakdown.iter().take(J_LAW_MAX_TIERS).enumerate() {
        out.breakdown[i].base_amount = step.base_amount;
        out.breakdown[i].rate_numer = step.rate_numer;
        out.breakdown[i].rate_denom = step.rate_denom;
        out.breakdown[i].result = step.result.as_yen();
        copy_str_to_fixed_buf(&step.label, &mut out.breakdown[i].label);
    }

    0
}

// ─── 源泉徴収 C 互換型定義 ──────────────────────────────────────────────────────

/// 源泉徴収税額の計算結果（C 互換）。
#[repr(C)]
pub struct JLawWithholdingTaxResult {
    /// 支払総額（円）。
    pub gross_payment_amount: u64,
    /// 源泉徴収税額の計算対象額（円）。
    pub taxable_payment_amount: u64,
    /// 源泉徴収税額（円）。
    pub tax_amount: u64,
    /// 源泉徴収後の支払額（円）。
    pub net_payment_amount: u64,
    /// カテゴリコード。
    pub category: u32,
    /// 応募作品等の入選賞金・謝金の非課税特例を適用したか。
    pub submission_prize_exempted: c_int,
    /// 計算内訳。
    pub breakdown: [JLawBreakdownStep; J_LAW_MAX_TIERS],
    /// breakdown の有効件数。
    pub breakdown_len: c_int,
}

/// 所得税法第204条第1項に基づく報酬・料金等の源泉徴収税額を計算する。
///
/// # 法的根拠
/// 所得税法 第204条第1項 / 国税庁タックスアンサー No.2795 / No.2798 / No.2810
///
/// # 引数
/// - `payment_amount`: 支払総額（円）
/// - `separated_consumption_tax_amount`: 区分表示された消費税額（円）
/// - `year`, `month`, `day`: 基準日
/// - `category`: カテゴリコード（`J_LAW_WITHHOLDING_TAX_CATEGORY_*`）
/// - `is_submission_prize`: 応募作品等の入選賞金・謝金として扱うか
/// - `out_result`: [OUT] 計算結果の書き込み先
/// - `error_buf`: [OUT] エラーメッセージの書き込み先
/// - `error_buf_len`: `error_buf` のバイト長
///
/// # Safety
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_withholding_tax(
    payment_amount: u64,
    separated_consumption_tax_amount: u64,
    year: u16,
    month: u8,
    day: u8,
    category: u32,
    is_submission_prize: c_int,
    out_result: *mut JLawWithholdingTaxResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    let category = match WithholdingTaxCategory::from_ffi_code(category) {
        Ok(category) => category,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    let params = match load_withholding_tax_params(LegalDate::new(year, month, day)) {
        Ok(params) => params,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    let mut flags = HashSet::new();
    if is_submission_prize != 0 {
        flags.insert(WithholdingTaxFlag::IsSubmissionPrize);
    }

    let ctx = WithholdingTaxContext {
        payment_amount,
        separated_consumption_tax_amount,
        category,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardWithholdingTaxPolicy),
    };

    let result = match calculate_withholding_tax(&ctx, &params) {
        Ok(result) => result,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    let out = &mut *out_result;
    out.gross_payment_amount = result.gross_payment_amount.as_yen();
    out.taxable_payment_amount = result.taxable_payment_amount.as_yen();
    out.tax_amount = result.tax_amount.as_yen();
    out.net_payment_amount = result.net_payment_amount.as_yen();
    out.category = u32::from(result.category);
    out.submission_prize_exempted = if result.submission_prize_exempted {
        1
    } else {
        0
    };
    out.breakdown_len = write_breakdown(&result.breakdown, &mut out.breakdown);

    0
}

// ─── 所得税 C 互換型定義 ────────────────────────────────────────────────────────

/// 所得税の計算内訳（速算表の適用結果・C 互換）。
#[repr(C)]
pub struct JLawIncomeTaxStep {
    /// ラベル（NUL 終端・最大 63 文字）。
    pub label: [c_char; J_LAW_LABEL_LEN],
    /// 課税所得金額（円）。
    pub taxable_income: u64,
    pub rate_numer: u64,
    pub rate_denom: u64,
    /// 速算表の控除額（円）。
    pub deduction: u64,
    /// 算出税額（円）。
    pub result: u64,
}

/// 所得税の計算結果（C 互換）。
#[repr(C)]
pub struct JLawIncomeTaxResult {
    /// 基準所得税額（円）。
    pub base_tax: u64,
    /// 復興特別所得税額（円）。
    pub reconstruction_tax: u64,
    /// 申告納税額（円・100円未満切り捨て）。
    pub total_tax: u64,
    /// 復興特別所得税が適用されたか（0 = false, 1 = true）。
    pub reconstruction_tax_applied: c_int,
    /// 計算内訳。
    pub breakdown: [JLawIncomeTaxStep; J_LAW_MAX_TIERS],
    /// breakdown の有効件数。
    pub breakdown_len: c_int,
}

/// 所得控除の内訳1行（C 互換）。
#[repr(C)]
pub struct JLawIncomeDeductionLine {
    /// 控除種別定数。
    pub kind: u32,
    /// ラベル（NUL 終端・最大 63 文字）。
    pub label: [c_char; J_LAW_LABEL_LEN],
    /// 控除額（円）。
    pub amount: u64,
}

/// 所得控除計算の入力（C 互換）。
#[repr(C)]
pub struct JLawIncomeDeductionInput {
    /// 総所得金額等（円）。
    pub total_income_amount: u64,
    /// 基準日（年）。
    pub year: u16,
    /// 基準日（月）。
    pub month: u8,
    /// 基準日（日）。
    pub day: u8,
    /// 配偶者控除入力があるか。
    pub has_spouse: c_int,
    /// 配偶者の合計所得金額（円）。
    pub spouse_total_income_amount: u64,
    /// 配偶者が生計を一にするか。
    pub spouse_is_same_household: c_int,
    /// 配偶者が老人控除対象配偶者か。
    pub spouse_is_elderly: c_int,
    /// 一般の控除対象扶養親族の人数。
    pub dependent_general_count: u64,
    /// 特定扶養親族の人数。
    pub dependent_specific_count: u64,
    /// 同居老親等の人数。
    pub dependent_elderly_cohabiting_count: u64,
    /// 同居老親等以外の老人扶養親族の人数。
    pub dependent_elderly_other_count: u64,
    /// 社会保険料控除の対象支払額（円）。
    pub social_insurance_premium_paid: u64,
    /// 医療費控除入力があるか。
    pub has_medical: c_int,
    /// 支払医療費（円）。
    pub medical_expense_paid: u64,
    /// 補填額（円）。
    pub medical_reimbursed_amount: u64,
    /// 生命保険料控除入力があるか。
    pub has_life_insurance: c_int,
    /// 新契約の一般生命保険料（円）。
    pub life_new_general_paid_amount: u64,
    /// 新契約の個人年金保険料（円）。
    pub life_new_individual_pension_paid_amount: u64,
    /// 新契約の介護医療保険料（円）。
    pub life_new_care_medical_paid_amount: u64,
    /// 旧契約の一般生命保険料（円）。
    pub life_old_general_paid_amount: u64,
    /// 旧契約の個人年金保険料（円）。
    pub life_old_individual_pension_paid_amount: u64,
    /// 寄附金控除入力があるか。
    pub has_donation: c_int,
    /// 控除対象寄附金額（円）。
    pub donation_qualified_amount: u64,
}

/// 所得控除の計算結果（C 互換）。
#[repr(C)]
pub struct JLawIncomeDeductionResult {
    /// 総所得金額等（円）。
    pub total_income_amount: u64,
    /// 所得控除額合計（円）。
    pub total_deductions: u64,
    /// 1,000円未満切り捨て前の課税所得金額（円）。
    pub taxable_income_before_truncation: u64,
    /// 1,000円未満切り捨て後の課税所得金額（円）。
    pub taxable_income: u64,
    /// 控除内訳。
    pub breakdown: [JLawIncomeDeductionLine; J_LAW_MAX_DEDUCTION_LINES],
    /// breakdown の有効件数。
    pub breakdown_len: c_int,
}

/// 所得控除から所得税額までの通し計算結果（C 互換）。
#[repr(C)]
pub struct JLawIncomeTaxAssessmentResult {
    /// 総所得金額等（円）。
    pub total_income_amount: u64,
    /// 所得控除額合計（円）。
    pub total_deductions: u64,
    /// 1,000円未満切り捨て前の課税所得金額（円）。
    pub taxable_income_before_truncation: u64,
    /// 1,000円未満切り捨て後の課税所得金額（円）。
    pub taxable_income: u64,
    /// 基準所得税額（円）。
    pub base_tax: u64,
    /// 復興特別所得税額（円）。
    pub reconstruction_tax: u64,
    /// 申告納税額（円）。
    pub total_tax: u64,
    /// 復興特別所得税が適用されたか。
    pub reconstruction_tax_applied: c_int,
    /// 所得控除の内訳。
    pub deduction_breakdown: [JLawIncomeDeductionLine; J_LAW_MAX_DEDUCTION_LINES],
    /// deduction_breakdown の有効件数。
    pub deduction_breakdown_len: c_int,
    /// 所得税の内訳。
    pub tax_breakdown: [JLawIncomeTaxStep; J_LAW_MAX_TIERS],
    /// tax_breakdown の有効件数。
    pub tax_breakdown_len: c_int,
}

// ─── 所得税 C FFI 公開関数 ──────────────────────────────────────────────────────

/// 所得税法第89条に基づく所得税額を計算する。
///
/// # 法的根拠
/// 所得税法 第89条第1項 / 復興財源確保法 第13条
///
/// # 引数
/// - `taxable_income`: 課税所得金額（円）
/// - `year`, `month`, `day`: 基準日
/// - `apply_reconstruction_tax`: 復興特別所得税を適用するか（0 = false, 非0 = true）
/// - `out_result`: [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
/// - `error_buf`: [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
/// - `error_buf_len`: `error_buf` のバイト長（推奨: `J_LAW_ERROR_BUF_LEN` = 256）
///
/// # 戻り値
/// - `0`: 成功。`out_result` にデータが書き込まれている。
/// - `非0`: 失敗。`error_buf` に NUL 終端エラーメッセージが書き込まれている。
///
/// # Safety
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
/// - `error_buf_len` は 1 以上であること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_income_tax(
    taxable_income: u64,
    year: u16,
    month: u8,
    day: u8,
    apply_reconstruction_tax: c_int,
    out_result: *mut JLawIncomeTaxResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    // パラメータロード
    let params = match load_income_tax_params(LegalDate::new(year, month, day)) {
        Ok(p) => p,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // フラグ構築
    let mut flags = HashSet::new();
    if apply_reconstruction_tax != 0 {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }

    let ctx = IncomeTaxContext {
        taxable_income,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };

    // 計算実行
    let result = match calculate_income_tax(&ctx, &params) {
        Ok(r) => r,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // 結果を out_result に書き込む
    let out = &mut *out_result;
    out.base_tax = result.base_tax.as_yen();
    out.reconstruction_tax = result.reconstruction_tax.as_yen();
    out.total_tax = result.total_tax.as_yen();
    out.reconstruction_tax_applied = if result.reconstruction_tax_applied {
        1
    } else {
        0
    };
    out.breakdown_len = write_income_tax_breakdown(&result.breakdown, &mut out.breakdown);

    0
}

/// 所得控除を計算し、課税所得金額までを返す。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
///
/// # Safety
/// - `input` は有効な入力ポインタであること。
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_income_deductions(
    input: *const JLawIncomeDeductionInput,
    out_result: *mut JLawIncomeDeductionResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if input.is_null() || out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    let input = &*input;
    let ctx = match to_income_deduction_context(input) {
        Ok(ctx) => ctx,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };
    let params = match load_income_tax_deduction_params(LegalDate::new(
        input.year,
        input.month,
        input.day,
    )) {
        Ok(params) => params,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };
    let result = match calculate_income_deductions(&ctx, &params) {
        Ok(result) => result,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    let out = &mut *out_result;
    out.total_income_amount = result.total_income_amount.as_yen();
    out.total_deductions = result.total_deductions.as_yen();
    out.taxable_income_before_truncation = result.taxable_income_before_truncation.as_yen();
    out.taxable_income = result.taxable_income.as_yen();
    out.breakdown_len = write_income_deduction_breakdown(&result.breakdown, &mut out.breakdown);

    0
}

/// 所得控除の計算から所得税額までを通しで計算する。
///
/// # 法的根拠
/// 所得税法 第73条（医療費控除）
/// 所得税法 第74条（社会保険料控除）
/// 所得税法 第76条（生命保険料控除）
/// 所得税法 第78条（寄附金控除）
/// 所得税法 第83条（配偶者控除）
/// 所得税法 第84条（扶養控除）
/// 所得税法 第86条（基礎控除）
/// 所得税法 第89条第1項（所得税の税率）
/// 復興財源確保法 第13条（復興特別所得税）
///
/// # Safety
/// - `input` は有効な入力ポインタであること。
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_income_tax_assessment(
    input: *const JLawIncomeDeductionInput,
    apply_reconstruction_tax: c_int,
    out_result: *mut JLawIncomeTaxAssessmentResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if input.is_null() || out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    let input = &*input;
    let deduction_context = match to_income_deduction_context(input) {
        Ok(ctx) => ctx,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };
    let deduction_params = match load_income_tax_deduction_params(LegalDate::new(
        input.year,
        input.month,
        input.day,
    )) {
        Ok(params) => params,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };
    let tax_params =
        match load_income_tax_params(LegalDate::new(input.year, input.month, input.day)) {
            Ok(params) => params,
            Err(e) => {
                write_error_msg(&e.to_string(), error_buf, error_buf_len);
                return 1;
            }
        };

    let mut flags = HashSet::new();
    if apply_reconstruction_tax != 0 {
        flags.insert(IncomeTaxFlag::ApplyReconstructionTax);
    }
    let ctx = IncomeTaxAssessmentContext {
        deduction_context,
        flags,
        policy: Box::new(StandardIncomeTaxPolicy),
    };
    let result = match calculate_income_tax_assessment(&ctx, &deduction_params, &tax_params) {
        Ok(result) => result,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    let out = &mut *out_result;
    out.total_income_amount = result.deductions.total_income_amount.as_yen();
    out.total_deductions = result.deductions.total_deductions.as_yen();
    out.taxable_income_before_truncation =
        result.deductions.taxable_income_before_truncation.as_yen();
    out.taxable_income = result.deductions.taxable_income.as_yen();
    out.base_tax = result.tax.base_tax.as_yen();
    out.reconstruction_tax = result.tax.reconstruction_tax.as_yen();
    out.total_tax = result.tax.total_tax.as_yen();
    out.reconstruction_tax_applied = if result.tax.reconstruction_tax_applied {
        1
    } else {
        0
    };
    out.deduction_breakdown_len = write_income_deduction_breakdown(
        &result.deductions.breakdown,
        &mut out.deduction_breakdown,
    );
    out.tax_breakdown_len =
        write_income_tax_breakdown(&result.tax.breakdown, &mut out.tax_breakdown);

    0
}

// ─── 消費税 C 互換型定義 ────────────────────────────────────────────────────────

/// 消費税の計算結果（C 互換）。
#[repr(C)]
pub struct JLawConsumptionTaxResult {
    /// 消費税額（円）。
    pub tax_amount: u64,
    /// 税込金額（円）。
    pub amount_with_tax: u64,
    /// 税抜金額（円）。
    pub amount_without_tax: u64,
    /// 適用税率の分子。
    pub applied_rate_numer: u64,
    /// 適用税率の分母。
    pub applied_rate_denom: u64,
    /// 軽減税率が適用されたか（0 = false, 1 = true）。
    pub is_reduced_rate: c_int,
}

// ─── 消費税 C FFI 公開関数 ──────────────────────────────────────────────────────

/// 消費税法第29条に基づく消費税額を計算する。
///
/// # 法的根拠
/// 消費税法 第29条（税率）
///
/// # 引数
/// - `amount`: 課税標準額（税抜き・円）
/// - `year`, `month`, `day`: 基準日
/// - `is_reduced_rate`: 軽減税率フラグ（0 = false, 非0 = true）
///   2019-10-01以降の飲食料品・新聞等に適用される8%軽減税率。
///   WARNING: 事実認定は呼び出し元の責任。
/// - `out_result`: [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
/// - `error_buf`: [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
/// - `error_buf_len`: `error_buf` のバイト長（推奨: `J_LAW_ERROR_BUF_LEN` = 256）
///
/// # 戻り値
/// - `0`: 成功。`out_result` にデータが書き込まれている。
/// - `非0`: 失敗。`error_buf` に NUL 終端エラーメッセージが書き込まれている。
///
/// # Safety
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
/// - `error_buf_len` は 1 以上であること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_consumption_tax(
    amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate: c_int,
    out_result: *mut JLawConsumptionTaxResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    // パラメータロード
    let params = match load_consumption_tax_params(LegalDate::new(year, month, day)) {
        Ok(p) => p,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // フラグ構築
    let mut flags = HashSet::new();
    if is_reduced_rate != 0 {
        flags.insert(ConsumptionTaxFlag::ReducedRate);
    }

    let ctx = ConsumptionTaxContext {
        amount,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardConsumptionTaxPolicy),
    };

    // 計算実行
    let result = match calculate_consumption_tax(&ctx, &params) {
        Ok(r) => r,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // 結果を out_result に書き込む
    let out = &mut *out_result;
    out.tax_amount = result.tax_amount.as_yen();
    out.amount_with_tax = result.amount_with_tax.as_yen();
    out.amount_without_tax = result.amount_without_tax.as_yen();
    out.applied_rate_numer = result.applied_rate_numer;
    out.applied_rate_denom = result.applied_rate_denom;
    out.is_reduced_rate = if result.is_reduced_rate { 1 } else { 0 };

    0
}

// ─── 印紙税 C 互換型定義 ────────────────────────────────────────────────────────

/// 印紙税の計算結果（C 互換）。
#[repr(C)]
pub struct JLawStampTaxResult {
    /// 印紙税額（円）。
    pub tax_amount: u64,
    /// 適用されたブラケットの表示名（NUL 終端・最大 63 文字）。
    pub bracket_label: [c_char; J_LAW_LABEL_LEN],
    /// 軽減税率が適用されたか（0 = false, 1 = true）。
    pub reduced_rate_applied: c_int,
}

// ─── 印紙税 C FFI 公開関数 ──────────────────────────────────────────────────────

/// 印紙税法 別表第一に基づく印紙税額を計算する。
///
/// # 法的根拠
/// 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条
///
/// # 引数
/// - `contract_amount`: 契約金額（円）
/// - `year`, `month`, `day`: 契約書作成日
/// - `is_reduced_rate_applicable`: 軽減税率適用フラグ（0 = false, 非0 = true）
/// - `out_result`: [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
/// - `error_buf`: [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
/// - `error_buf_len`: `error_buf` のバイト長（推奨: `J_LAW_ERROR_BUF_LEN` = 256）
///
/// # 戻り値
/// - `0`: 成功。`out_result` にデータが書き込まれている。
/// - `非0`: 失敗。`error_buf` に NUL 終端エラーメッセージが書き込まれている。
///
/// # Safety
/// - `out_result` は呼び出し元が所有する有効なポインタであること。
/// - `error_buf` は `error_buf_len` バイト以上の領域を指していること。
/// - `error_buf_len` は 1 以上であること。
#[no_mangle]
pub unsafe extern "C" fn j_law_calc_stamp_tax(
    contract_amount: u64,
    year: u16,
    month: u8,
    day: u8,
    is_reduced_rate_applicable: c_int,
    out_result: *mut JLawStampTaxResult,
    error_buf: *mut c_char,
    error_buf_len: c_int,
) -> c_int {
    if out_result.is_null() || error_buf.is_null() || error_buf_len <= 0 {
        return -1;
    }

    // パラメータロード
    let params = match load_stamp_tax_params(LegalDate::new(year, month, day)) {
        Ok(p) => p,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // フラグ構築
    let mut flags = HashSet::new();
    if is_reduced_rate_applicable != 0 {
        flags.insert(StampTaxFlag::IsReducedTaxRateApplicable);
    }

    let ctx = StampTaxContext {
        contract_amount,
        target_date: LegalDate::new(year, month, day),
        flags,
        policy: Box::new(StandardNtaPolicy),
    };

    // 計算実行
    let result = match calculate_stamp_tax(&ctx, &params) {
        Ok(r) => r,
        Err(e) => {
            write_error_msg(&e.to_string(), error_buf, error_buf_len);
            return 1;
        }
    };

    // 結果を out_result に書き込む
    let out = &mut *out_result;
    out.tax_amount = result.tax_amount.as_yen();
    copy_str_to_fixed_buf(&result.bracket_label, &mut out.bracket_label);
    out.reduced_rate_applied = if result.reduced_rate_applied { 1 } else { 0 };

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_buf_to_string(buf: &[c_char; J_LAW_LABEL_LEN]) -> String {
        let mut bytes = Vec::new();
        for &ch in buf {
            if ch == 0 {
                break;
            }
            bytes.push(ch as u8);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn error_buf_to_string(buf: &[c_char; J_LAW_ERROR_BUF_LEN]) -> String {
        let mut bytes = Vec::new();
        for &ch in buf {
            if ch == 0 {
                break;
            }
            bytes.push(ch as u8);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn sample_income_deduction_input() -> JLawIncomeDeductionInput {
        JLawIncomeDeductionInput {
            total_income_amount: 6_000_000,
            year: 2024,
            month: 1,
            day: 1,
            has_spouse: 0,
            spouse_total_income_amount: 0,
            spouse_is_same_household: 0,
            spouse_is_elderly: 0,
            dependent_general_count: 0,
            dependent_specific_count: 0,
            dependent_elderly_cohabiting_count: 0,
            dependent_elderly_other_count: 0,
            social_insurance_premium_paid: 150_000,
            has_medical: 1,
            medical_expense_paid: 500_000,
            medical_reimbursed_amount: 50_000,
            has_life_insurance: 1,
            life_new_general_paid_amount: 100_000,
            life_new_individual_pension_paid_amount: 60_000,
            life_new_care_medical_paid_amount: 80_000,
            life_old_general_paid_amount: 0,
            life_old_individual_pension_paid_amount: 0,
            has_donation: 1,
            donation_qualified_amount: 500_000,
        }
    }

    #[test]
    fn ffi_version_matches_constant() {
        assert_eq!(j_law_c_ffi_version(), J_LAW_C_FFI_VERSION);
    }

    #[test]
    fn brokerage_fee_writes_expected_c_result() {
        let mut result = unsafe { std::mem::zeroed::<JLawBrokerageFeeResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_brokerage_fee(
                5_000_000,
                2024,
                8,
                1,
                0,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.total_without_tax, 210_000);
        assert_eq!(result.tax_amount, 21_000);
        assert_eq!(result.total_with_tax, 231_000);
        assert_eq!(result.low_cost_special_applied, 0);
        assert_eq!(result.breakdown_len, 3);
        assert_eq!(fixed_buf_to_string(&result.breakdown[0].label), "tier1");
        assert_eq!(result.breakdown[0].base_amount, 2_000_000);
        assert_eq!(result.breakdown[0].result, 100_000);
        assert_eq!(fixed_buf_to_string(&result.breakdown[2].label), "tier3");
        assert_eq!(result.breakdown[2].base_amount, 1_000_000);
        assert_eq!(result.breakdown[2].result, 30_000);
    }

    #[test]
    fn brokerage_fee_propagates_errors() {
        let mut result = unsafe { std::mem::zeroed::<JLawBrokerageFeeResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_brokerage_fee(
                5_000_000,
                1970,
                11,
                30,
                0,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(error_buf_to_string(&error_buf).contains("1970-11-30"));
    }

    #[test]
    fn withholding_tax_writes_expected_c_result() {
        let mut result = unsafe { std::mem::zeroed::<JLawWithholdingTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_withholding_tax(
                1_500_000,
                0,
                2026,
                1,
                1,
                J_LAW_WITHHOLDING_TAX_CATEGORY_PROFESSIONAL_FEE,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.gross_payment_amount, 1_500_000);
        assert_eq!(result.taxable_payment_amount, 1_500_000);
        assert_eq!(result.tax_amount, 204_200);
        assert_eq!(result.net_payment_amount, 1_295_800);
        assert_eq!(
            result.category,
            J_LAW_WITHHOLDING_TAX_CATEGORY_PROFESSIONAL_FEE
        );
        assert_eq!(result.submission_prize_exempted, 0);
        assert_eq!(result.breakdown_len, 2);
        assert_eq!(
            fixed_buf_to_string(&result.breakdown[0].label),
            "1000000円以下の部分"
        );
    }

    #[test]
    fn withholding_tax_propagates_errors() {
        let mut result = unsafe { std::mem::zeroed::<JLawWithholdingTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_withholding_tax(
                100_000,
                100_001,
                2026,
                1,
                1,
                J_LAW_WITHHOLDING_TAX_CATEGORY_MANUSCRIPT_AND_LECTURE,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(error_buf_to_string(&error_buf).contains("separated_consumption_tax_amount"));
    }

    #[test]
    fn income_tax_writes_expected_c_result() {
        let mut result = unsafe { std::mem::zeroed::<JLawIncomeTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_income_tax(
                5_000_000,
                2024,
                1,
                1,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.base_tax, 572_500);
        assert_eq!(result.reconstruction_tax, 12_022);
        assert_eq!(result.total_tax, 584_500);
        assert_eq!(result.reconstruction_tax_applied, 1);
        assert_eq!(result.breakdown_len, 1);
        assert_eq!(
            fixed_buf_to_string(&result.breakdown[0].label),
            "330万円超695万円以下"
        );
        assert_eq!(result.breakdown[0].taxable_income, 5_000_000);
        assert_eq!(result.breakdown[0].deduction, 427_500);
        assert_eq!(result.breakdown[0].result, 572_500);
    }

    #[test]
    fn income_tax_propagates_errors() {
        let mut result = unsafe { std::mem::zeroed::<JLawIncomeTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_income_tax(
                5_000_000,
                1988,
                12,
                31,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(error_buf_to_string(&error_buf).contains("1988-12-31"));
    }

    #[test]
    fn income_deductions_write_expected_c_result() {
        let input = sample_income_deduction_input();
        let mut result = unsafe { std::mem::zeroed::<JLawIncomeDeductionResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_income_deductions(
                &input,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.total_income_amount, 6_000_000);
        assert_eq!(result.total_deductions, 1_593_000);
        assert_eq!(result.taxable_income_before_truncation, 4_407_000);
        assert_eq!(result.taxable_income, 4_407_000);
        assert_eq!(result.breakdown_len, 7);
        assert_eq!(result.breakdown[0].kind, 1);
        assert_eq!(fixed_buf_to_string(&result.breakdown[0].label), "基礎控除");
        assert_eq!(result.breakdown[4].amount, 350_000);
        assert_eq!(result.breakdown[5].amount, 115_000);
        assert_eq!(result.breakdown[6].amount, 498_000);
    }

    #[test]
    fn income_tax_assessment_writes_expected_c_result() {
        let input = sample_income_deduction_input();
        let mut result = unsafe { std::mem::zeroed::<JLawIncomeTaxAssessmentResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_income_tax_assessment(
                &input,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.total_deductions, 1_593_000);
        assert_eq!(result.taxable_income, 4_407_000);
        assert_eq!(result.base_tax, 453_900);
        assert_eq!(result.reconstruction_tax, 9_531);
        assert_eq!(result.total_tax, 463_400);
        assert_eq!(result.reconstruction_tax_applied, 1);
        assert_eq!(result.deduction_breakdown_len, 7);
        assert_eq!(result.tax_breakdown_len, 1);
    }

    #[test]
    fn consumption_tax_writes_expected_c_result() {
        let mut result = unsafe { std::mem::zeroed::<JLawConsumptionTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_consumption_tax(
                100_000,
                2024,
                1,
                1,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.tax_amount, 10_000);
        assert_eq!(result.amount_with_tax, 110_000);
        assert_eq!(result.amount_without_tax, 100_000);
        assert_eq!(result.applied_rate_numer, 10);
        assert_eq!(result.applied_rate_denom, 100);
        assert_eq!(result.is_reduced_rate, 0);
    }

    #[test]
    fn consumption_tax_propagates_errors() {
        let mut result = unsafe { std::mem::zeroed::<JLawConsumptionTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_consumption_tax(
                100_000,
                2016,
                1,
                1,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(
            error_buf_to_string(&error_buf).contains("軽減税率"),
            "unexpected error: {}",
            error_buf_to_string(&error_buf)
        );
    }

    #[test]
    fn stamp_tax_writes_expected_c_result() {
        let mut result = unsafe { std::mem::zeroed::<JLawStampTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_stamp_tax(
                5_000_000,
                2024,
                8,
                1,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(error_buf_to_string(&error_buf), "");
        assert_eq!(result.tax_amount, 1_000);
        assert_eq!(fixed_buf_to_string(&result.bracket_label), "500万円以下");
        assert_eq!(result.reduced_rate_applied, 1);
    }

    #[test]
    fn stamp_tax_propagates_errors() {
        let mut result = unsafe { std::mem::zeroed::<JLawStampTaxResult>() };
        let mut error_buf = [0; J_LAW_ERROR_BUF_LEN];

        let status = unsafe {
            j_law_calc_stamp_tax(
                5_000_000,
                2014,
                3,
                31,
                0,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(error_buf_to_string(&error_buf).contains("2014-03-31"));
    }
}
