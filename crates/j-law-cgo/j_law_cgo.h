#ifndef J_LAW_CGO_H
#define J_LAW_CGO_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ─── 定数 ───────────────────────────────────────────────────────────────── */

/** ティア内訳の最大件数。現行法令では 3 ティアだが余裕を持たせている。 */
#define J_LAW_MAX_TIERS 8

/** ティアラベルの最大バイト長（NUL 終端含む）。 */
#define J_LAW_LABEL_LEN 64

/** エラーバッファの推奨バイト長。 */
#define J_LAW_ERROR_BUF_LEN 256

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

#ifdef __cplusplus
}
#endif

#endif /* J_LAW_CGO_H */
