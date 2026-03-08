use std::collections::HashSet;
use std::os::raw::{c_char, c_int};

use j_law_core::domains::consumption_tax::{
    calculator::calculate_consumption_tax,
    context::{ConsumptionTaxContext, ConsumptionTaxFlag},
    policy::StandardConsumptionTaxPolicy,
};
use j_law_core::domains::income_tax::{
    calculator::calculate_income_tax,
    context::{IncomeTaxContext, IncomeTaxFlag},
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
use j_law_core::LegalDate;
use j_law_registry::load_brokerage_fee_params;
use j_law_registry::load_consumption_tax_params;
use j_law_registry::load_income_tax_params;
use j_law_registry::load_stamp_tax_params;

// ─── 定数 ─────────────────────────────────────────────────────────────────────

/// ティア内訳の最大件数。現行法令では 3 ティアだが余裕を持たせる。
pub const J_LAW_MAX_TIERS: usize = 8;

/// ティアラベルの最大バイト長（NUL 終端含む）。
pub const J_LAW_LABEL_LEN: usize = 64;

/// エラーバッファのデフォルトバイト長。Go 側のアロケーション目安。
pub const J_LAW_ERROR_BUF_LEN: usize = 256;

/// Go / Ruby 向け C ABI の互換バージョン。
pub const J_LAW_FFI_ABI_VERSION: u32 = 1;

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

// ─── C FFI 公開関数 ────────────────────────────────────────────────────────────

/// j-law-ffi の ABI バージョンを返す。
///
/// # 法的根拠
/// なし（FFI 互換確認用）
#[no_mangle]
pub extern "C" fn j_law_ffi_abi_version() -> u32 {
    J_LAW_FFI_ABI_VERSION
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
    out.breakdown_len = result.breakdown.len().min(J_LAW_MAX_TIERS) as c_int;

    for (i, step) in result.breakdown.iter().take(J_LAW_MAX_TIERS).enumerate() {
        out.breakdown[i].taxable_income = step.taxable_income;
        out.breakdown[i].rate_numer = step.rate_numer;
        out.breakdown[i].rate_denom = step.rate_denom;
        out.breakdown[i].deduction = step.deduction;
        out.breakdown[i].result = step.result.as_yen();
        copy_str_to_fixed_buf(&step.label, &mut out.breakdown[i].label);
    }

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

    #[test]
    fn abi_version_matches_constant() {
        assert_eq!(j_law_ffi_abi_version(), J_LAW_FFI_ABI_VERSION);
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
                2014,
                12,
                31,
                1,
                &mut result,
                error_buf.as_mut_ptr(),
                J_LAW_ERROR_BUF_LEN as c_int,
            )
        };

        assert_eq!(status, 1);
        assert!(error_buf_to_string(&error_buf).contains("2014-12-31"));
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
