#ifndef J_LAW_C_FFI_H
#define J_LAW_C_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ─── 定数 ───────────────────────────────────────────────────────────────── */

/** ティア内訳の最大件数。現行法令では 3 ティアだが余裕を持たせている。 */
#define J_LAW_MAX_TIERS 8

/** 所得控除内訳の最大件数。 */
#define J_LAW_MAX_DEDUCTION_LINES 8

/** ティアラベルの最大バイト長（NUL 終端含む）。 */
#define J_LAW_LABEL_LEN 64

/** エラーバッファの推奨バイト長。 */
#define J_LAW_ERROR_BUF_LEN 256

/** j-law-c-ffi の FFI 互換バージョン。 */
#define J_LAW_C_FFI_VERSION 3

/** 所得控除種別定数。 */
#define J_LAW_INCOME_DEDUCTION_KIND_BASIC 1
#define J_LAW_INCOME_DEDUCTION_KIND_SPOUSE 2
#define J_LAW_INCOME_DEDUCTION_KIND_DEPENDENT 3
#define J_LAW_INCOME_DEDUCTION_KIND_SOCIAL_INSURANCE 4
#define J_LAW_INCOME_DEDUCTION_KIND_MEDICAL 5
#define J_LAW_INCOME_DEDUCTION_KIND_LIFE_INSURANCE 6
#define J_LAW_INCOME_DEDUCTION_KIND_DONATION 7

/* ─── 構造体 ─────────────────────────────────────────────────────────────── */

/**
 * 1 ティアの計算内訳。
 */
typedef struct {
    /** ティアラベル（NUL 終端・最大 63 文字）。 */
    char     label[J_LAW_LABEL_LEN];
    /** ティア対象金額（円）。 */
    uint64_t base_amount;
    uint64_t rate_numer;
    uint64_t rate_denom;
    /** ティア計算結果（円・端数切捨て済み）。 */
    uint64_t result;
} JLawBreakdownStep;

/**
 * 媒介報酬の計算結果。
 */
typedef struct {
    /** 税抜合計額（円）。 */
    uint64_t total_without_tax;
    /** 税込合計額（円）。 */
    uint64_t total_with_tax;
    /** 消費税額（円）。 */
    uint64_t tax_amount;
    /** 低廉な空き家特例が適用されたか（0 = false, 1 = true）。 */
    int      low_cost_special_applied;
    /** 各ティアの計算内訳（breakdown_len 件が有効）。 */
    JLawBreakdownStep breakdown[J_LAW_MAX_TIERS];
    /** breakdown の有効件数。 */
    int      breakdown_len;
} JLawBrokerageFeeResult;

/* ─── 関数 ───────────────────────────────────────────────────────────────── */

/**
 * j-law-c-ffi の FFI バージョンを返す。
 *
 * Ruby / Go バインディングはロード時にこの値を確認し、
 * 期待する C FFI と一致するかを検証する。
 */
uint32_t j_law_c_ffi_version(void);

/**
 * 宅建業法第46条に基づく媒介報酬を計算する。
 *
 * 法的根拠: 宅地建物取引業法 第46条第1項 / 国土交通省告示
 *
 * @param price                    売買価格（円）
 * @param year                     基準日（年）
 * @param month                    基準日（月）
 * @param day                      基準日（日）
 * @param is_low_cost_vacant_house 低廉な空き家特例フラグ（0 = false, 非0 = true）
 *                                 WARNING: 事実認定は呼び出し元の責任。
 * @param is_seller                売主側フラグ（0 = false, 非0 = true）
 *                                 2018年1月1日〜2024年6月30日の低廉特例は売主のみ適用。
 *                                 WARNING: 売主・買主の事実認定は呼び出し元の責任。
 * @param out_result               [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
 * @param error_buf                [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
 * @param error_buf_len            error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                         成功時 0、失敗時 非0
 */
int j_law_calc_brokerage_fee(
    uint64_t price,
    uint16_t year,
    uint8_t  month,
    uint8_t  day,
    int      is_low_cost_vacant_house,
    int      is_seller,
    JLawBrokerageFeeResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/* ─── 所得税 構造体 ───────────────────────────────────────────────────────── */

/**
 * 所得税の計算内訳（速算表の適用結果）。
 */
typedef struct {
    /** ラベル（NUL 終端・最大 63 文字）。 */
    char     label[J_LAW_LABEL_LEN];
    /** 課税所得金額（円）。 */
    uint64_t taxable_income;
    uint64_t rate_numer;
    uint64_t rate_denom;
    /** 速算表の控除額（円）。 */
    uint64_t deduction;
    /** 算出税額（円）。 */
    uint64_t result;
} JLawIncomeTaxStep;

/**
 * 所得税の計算結果。
 */
typedef struct {
    /** 基準所得税額（円）。 */
    uint64_t base_tax;
    /** 復興特別所得税額（円）。 */
    uint64_t reconstruction_tax;
    /** 申告納税額（円・100円未満切り捨て）。 */
    uint64_t total_tax;
    /** 復興特別所得税が適用されたか（0 = false, 1 = true）。 */
    int      reconstruction_tax_applied;
    /** 計算内訳（breakdown_len 件が有効）。 */
    JLawIncomeTaxStep breakdown[J_LAW_MAX_TIERS];
    /** breakdown の有効件数。 */
    int      breakdown_len;
} JLawIncomeTaxResult;

/**
 * 所得控除の内訳1行。
 */
typedef struct {
    /** 控除種別定数（J_LAW_INCOME_DEDUCTION_KIND_*）。 */
    uint32_t kind;
    /** ラベル（NUL 終端・最大 63 文字）。 */
    char     label[J_LAW_LABEL_LEN];
    /** 控除額（円）。 */
    uint64_t amount;
} JLawIncomeDeductionLine;

/**
 * 所得控除計算の入力。
 */
typedef struct {
    uint64_t total_income_amount;
    uint16_t year;
    uint8_t  month;
    uint8_t  day;
    int      has_spouse;
    uint64_t spouse_total_income_amount;
    int      spouse_is_same_household;
    int      spouse_is_elderly;
    uint64_t dependent_general_count;
    uint64_t dependent_specific_count;
    uint64_t dependent_elderly_cohabiting_count;
    uint64_t dependent_elderly_other_count;
    uint64_t social_insurance_premium_paid;
    int      has_medical;
    uint64_t medical_expense_paid;
    uint64_t medical_reimbursed_amount;
    int      has_life_insurance;
    uint64_t life_new_general_paid_amount;
    uint64_t life_new_individual_pension_paid_amount;
    uint64_t life_new_care_medical_paid_amount;
    uint64_t life_old_general_paid_amount;
    uint64_t life_old_individual_pension_paid_amount;
    int      has_donation;
    uint64_t donation_qualified_amount;
} JLawIncomeDeductionInput;

/**
 * 所得控除の計算結果。
 */
typedef struct {
    uint64_t total_income_amount;
    uint64_t total_deductions;
    uint64_t taxable_income_before_truncation;
    uint64_t taxable_income;
    JLawIncomeDeductionLine breakdown[J_LAW_MAX_DEDUCTION_LINES];
    int      breakdown_len;
} JLawIncomeDeductionResult;

/**
 * 所得控除から所得税額までの通し計算結果。
 */
typedef struct {
    uint64_t total_income_amount;
    uint64_t total_deductions;
    uint64_t taxable_income_before_truncation;
    uint64_t taxable_income;
    uint64_t base_tax;
    uint64_t reconstruction_tax;
    uint64_t total_tax;
    int      reconstruction_tax_applied;
    JLawIncomeDeductionLine deduction_breakdown[J_LAW_MAX_DEDUCTION_LINES];
    int      deduction_breakdown_len;
    JLawIncomeTaxStep tax_breakdown[J_LAW_MAX_TIERS];
    int      tax_breakdown_len;
} JLawIncomeTaxAssessmentResult;

/* ─── 所得税 関数 ─────────────────────────────────────────────────────────── */

/**
 * 所得税法第89条に基づく所得税額を計算する。
 *
 * 法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条
 *
 * @param taxable_income          課税所得金額（円）
 * @param year                    対象年度（年）
 * @param month                   基準日（月）
 * @param day                     基準日（日）
 * @param apply_reconstruction_tax 復興特別所得税を適用するか（0 = false, 非0 = true）
 * @param out_result              [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
 * @param error_buf               [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
 * @param error_buf_len           error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                        成功時 0、失敗時 非0
 */
int j_law_calc_income_tax(
    uint64_t taxable_income,
    uint16_t year,
    uint8_t  month,
    uint8_t  day,
    int      apply_reconstruction_tax,
    JLawIncomeTaxResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/**
 * 所得控除額を計算し、課税所得金額までを返す。
 *
 * 法的根拠: 所得税法 第73条 / 第74条 / 第76条 / 第78条 / 第83条 / 第84条 / 第86条
 *
 * @param input                 [IN] 所得控除計算の入力
 * @param out_result            [OUT] 計算結果の書き込み先
 * @param error_buf             [OUT] エラーメッセージの書き込み先
 * @param error_buf_len         error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                      成功時 0、失敗時 非0
 */
int j_law_calc_income_deductions(
    const JLawIncomeDeductionInput *input,
    JLawIncomeDeductionResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/**
 * 所得控除から所得税額までを通しで計算する。
 *
 * 法的根拠: 所得税法 第73条 / 第74条 / 第76条 / 第78条 / 第83条 / 第84条 / 第86条 /
 *           第89条第1項 / 復興財源確保法 第13条
 *
 * @param input                    [IN] 所得控除計算の入力
 * @param apply_reconstruction_tax 復興特別所得税を適用するか（0 = false, 非0 = true）
 * @param out_result               [OUT] 計算結果の書き込み先
 * @param error_buf                [OUT] エラーメッセージの書き込み先
 * @param error_buf_len            error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                         成功時 0、失敗時 非0
 */
int j_law_calc_income_tax_assessment(
    const JLawIncomeDeductionInput *input,
    int      apply_reconstruction_tax,
    JLawIncomeTaxAssessmentResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/* ─── 消費税 構造体 ───────────────────────────────────────────────────────── */

/**
 * 消費税の計算結果。
 */
typedef struct {
    /** 消費税額（円）。 */
    uint64_t tax_amount;
    /** 税込金額（円）。 */
    uint64_t amount_with_tax;
    /** 税抜金額（円）。 */
    uint64_t amount_without_tax;
    /** 適用税率の分子。 */
    uint64_t applied_rate_numer;
    /** 適用税率の分母。 */
    uint64_t applied_rate_denom;
    /** 軽減税率が適用されたか（0 = false, 1 = true）。 */
    int      is_reduced_rate;
} JLawConsumptionTaxResult;

/* ─── 消費税 関数 ─────────────────────────────────────────────────────────── */

/**
 * 消費税法第29条に基づく消費税額を計算する。
 *
 * 法的根拠: 消費税法 第29条（税率）
 *
 * @param amount              課税標準額（税抜き・円）
 * @param year                基準日（年）
 * @param month               基準日（月）
 * @param day                 基準日（日）
 * @param is_reduced_rate     軽減税率フラグ（0 = false, 非0 = true）
 *                            2019-10-01以降の飲食料品・新聞等に適用される8%軽減税率。
 *                            WARNING: 事実認定は呼び出し元の責任。
 * @param out_result          [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
 * @param error_buf           [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
 * @param error_buf_len       error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                    成功時 0、失敗時 非0
 */
int j_law_calc_consumption_tax(
    uint64_t amount,
    uint16_t year,
    uint8_t  month,
    uint8_t  day,
    int      is_reduced_rate,
    JLawConsumptionTaxResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/* ─── 印紙税 構造体 ───────────────────────────────────────────────────────── */

/**
 * 印紙税の計算結果。
 */
typedef struct {
    /** 印紙税額（円）。 */
    uint64_t tax_amount;
    /** 適用されたブラケットの表示名（NUL 終端・最大 63 文字）。 */
    char     bracket_label[J_LAW_LABEL_LEN];
    /** 軽減税率が適用されたか（0 = false, 1 = true）。 */
    int      reduced_rate_applied;
} JLawStampTaxResult;

/**
 * 印紙税の文書種別。
 */
typedef enum {
    /** 不動産の譲渡に関する契約書（第1号文書）。 */
    J_LAW_STAMP_TAX_DOCUMENT_REAL_ESTATE_TRANSFER = 0,
    /** 建設工事の請負に関する契約書（第2号文書）。 */
    J_LAW_STAMP_TAX_DOCUMENT_CONSTRUCTION_CONTRACT = 1
} JLawStampTaxDocumentKind;

/* ─── 印紙税 関数 ─────────────────────────────────────────────────────────── */

/**
 * 印紙税法 別表第一に基づく印紙税額を計算する。
 *
 * 法的根拠: 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条
 *
 * @param contract_amount             契約金額（円）
 * @param year                        契約書作成日（年）
 * @param month                       契約書作成日（月）
 * @param day                         契約書作成日（日）
 * @param is_reduced_rate_applicable  軽減税率適用フラグ（0 = false, 非0 = true）
 *                                    WARNING: 事実認定は呼び出し元の責任。
 *                                    この関数は不動産譲渡契約書を既定値として扱います。
 * @param out_result                  [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
 * @param error_buf                   [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
 * @param error_buf_len               error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                            成功時 0、失敗時 非0
 */
int j_law_calc_stamp_tax(
    uint64_t contract_amount,
    uint16_t year,
    uint8_t  month,
    uint8_t  day,
    int      is_reduced_rate_applicable,
    JLawStampTaxResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

/**
 * 印紙税法 別表第一に基づく印紙税額を計算する（文書種別指定あり）。
 *
 * 法的根拠: 印紙税法 別表第一 第1号文書 / 第2号文書 / 租税特別措置法 第91条
 *
 * @param contract_amount             契約金額（円）
 * @param year                        契約書作成日（年）
 * @param month                       契約書作成日（月）
 * @param day                         契約書作成日（日）
 * @param is_reduced_rate_applicable  軽減税率適用フラグ（0 = false, 非0 = true）
 *                                    WARNING: 事実認定は呼び出し元の責任。
 * @param document_kind               文書種別（JLawStampTaxDocumentKind）
 * @param out_result                  [OUT] 計算結果の書き込み先（呼び出し元が確保すること）
 * @param error_buf                   [OUT] エラーメッセージの書き込み先（呼び出し元が確保すること）
 * @param error_buf_len               error_buf のバイト長（推奨: J_LAW_ERROR_BUF_LEN = 256）
 * @return                            成功時 0、失敗時 非0
 */
int j_law_calc_stamp_tax_with_document_kind(
    uint64_t contract_amount,
    uint16_t year,
    uint8_t  month,
    uint8_t  day,
    int      is_reduced_rate_applicable,
    int      document_kind,
    JLawStampTaxResult *out_result,
    char    *error_buf,
    int      error_buf_len
);

#ifdef __cplusplus
}
#endif

#endif /* J_LAW_C_FFI_H */
