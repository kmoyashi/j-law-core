#include <jni.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "j_law_c_ffi.h"

static void throw_runtime_exception(JNIEnv *env, const char *class_name, const char *message) {
    jclass exception_class = (*env)->FindClass(env, class_name);
    if (exception_class == NULL) {
        return;
    }
    (*env)->ThrowNew(env, exception_class, message);
}

static void throw_jlaw_exception(JNIEnv *env, const char *message) {
    throw_runtime_exception(env, "io/github/kmoyashi/jlaw/JLawException", message);
}

static void throw_illegal_argument(JNIEnv *env, const char *message) {
    throw_runtime_exception(env, "java/lang/IllegalArgumentException", message);
}

static int validate_u64(JNIEnv *env, jlong value, const char *field_name, uint64_t *out) {
    if (value < 0) {
        char buffer[128];
        snprintf(buffer, sizeof(buffer), "%s must be non-negative", field_name);
        throw_illegal_argument(env, buffer);
        return 0;
    }

    *out = (uint64_t)value;
    return 1;
}

static int validate_u16(JNIEnv *env, jint value, const char *field_name, uint16_t *out) {
    if (value < 0 || value > 65535) {
        char buffer[128];
        snprintf(buffer, sizeof(buffer), "%s must be between 0 and 65535", field_name);
        throw_illegal_argument(env, buffer);
        return 0;
    }

    *out = (uint16_t)value;
    return 1;
}

static int validate_u8(JNIEnv *env, jint value, const char *field_name, uint8_t *out) {
    if (value < 0 || value > 255) {
        char buffer[128];
        snprintf(buffer, sizeof(buffer), "%s must be between 0 and 255", field_name);
        throw_illegal_argument(env, buffer);
        return 0;
    }

    *out = (uint8_t)value;
    return 1;
}

static jstring new_string(JNIEnv *env, const char *value) {
    if (value == NULL || value[0] == '\0') {
        return NULL;
    }
    return (*env)->NewStringUTF(env, value);
}

static jobjectArray build_breakdown_array(JNIEnv *env, const JLawBreakdownStep *steps, jint length) {
    jclass step_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/BreakdownStep");
    if (step_class == NULL) {
        return NULL;
    }

    jmethodID ctor = (*env)->GetMethodID(env, step_class, "<init>", "(Ljava/lang/String;JJJJ)V");
    if (ctor == NULL) {
        return NULL;
    }

    if (length < 0) {
        length = 0;
    }
    if (length > J_LAW_MAX_TIERS) {
        length = J_LAW_MAX_TIERS;
    }

    jobjectArray array = (*env)->NewObjectArray(env, length, step_class, NULL);
    if (array == NULL) {
        return NULL;
    }

    for (jint i = 0; i < length; i++) {
        jstring label = new_string(env, steps[i].label);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }

        jobject step = (*env)->NewObject(
            env,
            step_class,
            ctor,
            label,
            (jlong)steps[i].base_amount,
            (jlong)steps[i].rate_numer,
            (jlong)steps[i].rate_denom,
            (jlong)steps[i].result
        );
        if (label != NULL) {
            (*env)->DeleteLocalRef(env, label);
        }
        if (step == NULL) {
            return NULL;
        }
        (*env)->SetObjectArrayElement(env, array, i, step);
        (*env)->DeleteLocalRef(env, step);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }
    }

    return array;
}

static jobjectArray build_income_tax_step_array(JNIEnv *env, const JLawIncomeTaxStep *steps, jint length) {
    jclass step_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeTaxStep");
    if (step_class == NULL) {
        return NULL;
    }

    jmethodID ctor = (*env)->GetMethodID(env, step_class, "<init>", "(Ljava/lang/String;JJJJJ)V");
    if (ctor == NULL) {
        return NULL;
    }

    if (length < 0) {
        length = 0;
    }
    if (length > J_LAW_MAX_TIERS) {
        length = J_LAW_MAX_TIERS;
    }

    jobjectArray array = (*env)->NewObjectArray(env, length, step_class, NULL);
    if (array == NULL) {
        return NULL;
    }

    for (jint i = 0; i < length; i++) {
        jstring label = new_string(env, steps[i].label);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }

        jobject step = (*env)->NewObject(
            env,
            step_class,
            ctor,
            label,
            (jlong)steps[i].taxable_income,
            (jlong)steps[i].rate_numer,
            (jlong)steps[i].rate_denom,
            (jlong)steps[i].deduction,
            (jlong)steps[i].result
        );
        if (label != NULL) {
            (*env)->DeleteLocalRef(env, label);
        }
        if (step == NULL) {
            return NULL;
        }
        (*env)->SetObjectArrayElement(env, array, i, step);
        (*env)->DeleteLocalRef(env, step);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }
    }

    return array;
}

static jobjectArray build_income_deduction_line_array(
    JNIEnv *env,
    const JLawIncomeDeductionLine *lines,
    jint length
) {
    jclass line_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeDeductionLine");
    if (line_class == NULL) {
        return NULL;
    }

    jmethodID ctor = (*env)->GetMethodID(env, line_class, "<init>", "(ILjava/lang/String;J)V");
    if (ctor == NULL) {
        return NULL;
    }

    if (length < 0) {
        length = 0;
    }
    if (length > J_LAW_MAX_DEDUCTION_LINES) {
        length = J_LAW_MAX_DEDUCTION_LINES;
    }

    jobjectArray array = (*env)->NewObjectArray(env, length, line_class, NULL);
    if (array == NULL) {
        return NULL;
    }

    for (jint i = 0; i < length; i++) {
        jstring label = new_string(env, lines[i].label);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }

        jobject line = (*env)->NewObject(
            env,
            line_class,
            ctor,
            (jint)lines[i].kind,
            label,
            (jlong)lines[i].amount
        );
        if (label != NULL) {
            (*env)->DeleteLocalRef(env, label);
        }
        if (line == NULL) {
            return NULL;
        }
        (*env)->SetObjectArrayElement(env, array, i, line);
        (*env)->DeleteLocalRef(env, line);
        if ((*env)->ExceptionCheck(env)) {
            return NULL;
        }
    }

    return array;
}

JNIEXPORT jint JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_ffiVersion(JNIEnv *env, jclass cls) {
    (void)env;
    (void)cls;
    return (jint)j_law_c_ffi_version();
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcBrokerageFee(
    JNIEnv *env,
    jclass cls,
    jlong price,
    jint year,
    jint month,
    jint day,
    jboolean is_low_cost_vacant_house,
    jboolean is_seller
) {
    (void)cls;
    uint64_t price_u64;
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, price, "price", &price_u64)
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)) {
        return NULL;
    }

    JLawBrokerageFeeResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_brokerage_fee(
        price_u64,
        year_u16,
        month_u8,
        day_u8,
        is_low_cost_vacant_house == JNI_TRUE ? 1 : 0,
        is_seller == JNI_TRUE ? 1 : 0,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    jobjectArray breakdown = build_breakdown_array(env, result.breakdown, result.breakdown_len);
    if (breakdown == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/BrokerageFeeResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(JJJZ[Lio/github/kmoyashi/jlaw/BreakdownStep;)V"
    );

    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result.total_without_tax,
        (jlong)result.total_with_tax,
        (jlong)result.tax_amount,
        result.low_cost_special_applied != 0 ? JNI_TRUE : JNI_FALSE,
        breakdown
    );
    if (breakdown != NULL) {
        (*env)->DeleteLocalRef(env, breakdown);
    }
    return output;
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcIncomeTax(
    JNIEnv *env,
    jclass cls,
    jlong taxable_income,
    jint year,
    jint month,
    jint day,
    jboolean apply_reconstruction_tax
) {
    (void)cls;
    uint64_t taxable_income_u64;
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, taxable_income, "taxableIncome", &taxable_income_u64)
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)) {
        return NULL;
    }

    JLawIncomeTaxResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_income_tax(
        taxable_income_u64,
        year_u16,
        month_u8,
        day_u8,
        apply_reconstruction_tax == JNI_TRUE ? 1 : 0,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    jobjectArray breakdown = build_income_tax_step_array(env, result.breakdown, result.breakdown_len);
    if (breakdown == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeTaxResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(JJJZ[Lio/github/kmoyashi/jlaw/IncomeTaxStep;)V"
    );

    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result.base_tax,
        (jlong)result.reconstruction_tax,
        (jlong)result.total_tax,
        result.reconstruction_tax_applied != 0 ? JNI_TRUE : JNI_FALSE,
        breakdown
    );
    if (breakdown != NULL) {
        (*env)->DeleteLocalRef(env, breakdown);
    }
    return output;
}

static int fill_income_deduction_input(
    JNIEnv *env,
    JLawIncomeDeductionInput *input,
    jlong total_income_amount,
    jint year,
    jint month,
    jint day,
    jboolean has_spouse,
    jlong spouse_total_income_amount,
    jboolean spouse_is_same_household,
    jboolean spouse_is_elderly,
    jlong dependent_general_count,
    jlong dependent_specific_count,
    jlong dependent_elderly_cohabiting_count,
    jlong dependent_elderly_other_count,
    jlong social_insurance_premium_paid,
    jboolean has_medical,
    jlong medical_expense_paid,
    jlong medical_reimbursed_amount,
    jboolean has_life_insurance,
    jlong life_new_general_paid_amount,
    jlong life_new_individual_pension_paid_amount,
    jlong life_new_care_medical_paid_amount,
    jlong life_old_general_paid_amount,
    jlong life_old_individual_pension_paid_amount,
    jboolean has_donation,
    jlong donation_qualified_amount
) {
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, total_income_amount, "totalIncomeAmount", &input->total_income_amount)
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)
        || !validate_u64(env, spouse_total_income_amount, "spouseTotalIncomeAmount", &input->spouse_total_income_amount)
        || !validate_u64(env, dependent_general_count, "dependentGeneralCount", &input->dependent_general_count)
        || !validate_u64(env, dependent_specific_count, "dependentSpecificCount", &input->dependent_specific_count)
        || !validate_u64(
            env,
            dependent_elderly_cohabiting_count,
            "dependentElderlyCohabitingCount",
            &input->dependent_elderly_cohabiting_count
        )
        || !validate_u64(
            env,
            dependent_elderly_other_count,
            "dependentElderlyOtherCount",
            &input->dependent_elderly_other_count
        )
        || !validate_u64(env, social_insurance_premium_paid, "socialInsurancePremiumPaid", &input->social_insurance_premium_paid)
        || !validate_u64(env, medical_expense_paid, "medicalExpensePaid", &input->medical_expense_paid)
        || !validate_u64(env, medical_reimbursed_amount, "medicalReimbursedAmount", &input->medical_reimbursed_amount)
        || !validate_u64(env, life_new_general_paid_amount, "lifeNewGeneralPaidAmount", &input->life_new_general_paid_amount)
        || !validate_u64(
            env,
            life_new_individual_pension_paid_amount,
            "lifeNewIndividualPensionPaidAmount",
            &input->life_new_individual_pension_paid_amount
        )
        || !validate_u64(
            env,
            life_new_care_medical_paid_amount,
            "lifeNewCareMedicalPaidAmount",
            &input->life_new_care_medical_paid_amount
        )
        || !validate_u64(env, life_old_general_paid_amount, "lifeOldGeneralPaidAmount", &input->life_old_general_paid_amount)
        || !validate_u64(
            env,
            life_old_individual_pension_paid_amount,
            "lifeOldIndividualPensionPaidAmount",
            &input->life_old_individual_pension_paid_amount
        )
        || !validate_u64(env, donation_qualified_amount, "donationQualifiedAmount", &input->donation_qualified_amount)) {
        return 0;
    }

    input->year = year_u16;
    input->month = month_u8;
    input->day = day_u8;
    input->has_spouse = has_spouse == JNI_TRUE ? 1 : 0;
    input->spouse_is_same_household = spouse_is_same_household == JNI_TRUE ? 1 : 0;
    input->spouse_is_elderly = spouse_is_elderly == JNI_TRUE ? 1 : 0;
    input->has_medical = has_medical == JNI_TRUE ? 1 : 0;
    input->has_life_insurance = has_life_insurance == JNI_TRUE ? 1 : 0;
    input->has_donation = has_donation == JNI_TRUE ? 1 : 0;

    return 1;
}

static jobject build_income_deduction_result(JNIEnv *env, const JLawIncomeDeductionResult *result) {
    jobjectArray breakdown = build_income_deduction_line_array(env, result->breakdown, result->breakdown_len);
    if (breakdown == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeDeductionResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(JJJJ[Lio/github/kmoyashi/jlaw/IncomeDeductionLine;)V"
    );
    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result->total_income_amount,
        (jlong)result->total_deductions,
        (jlong)result->taxable_income_before_truncation,
        (jlong)result->taxable_income,
        breakdown
    );
    if (breakdown != NULL) {
        (*env)->DeleteLocalRef(env, breakdown);
    }
    return output;
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcIncomeDeductions(
    JNIEnv *env,
    jclass cls,
    jlong total_income_amount,
    jint year,
    jint month,
    jint day,
    jboolean has_spouse,
    jlong spouse_total_income_amount,
    jboolean spouse_is_same_household,
    jboolean spouse_is_elderly,
    jlong dependent_general_count,
    jlong dependent_specific_count,
    jlong dependent_elderly_cohabiting_count,
    jlong dependent_elderly_other_count,
    jlong social_insurance_premium_paid,
    jboolean has_medical,
    jlong medical_expense_paid,
    jlong medical_reimbursed_amount,
    jboolean has_life_insurance,
    jlong life_new_general_paid_amount,
    jlong life_new_individual_pension_paid_amount,
    jlong life_new_care_medical_paid_amount,
    jlong life_old_general_paid_amount,
    jlong life_old_individual_pension_paid_amount,
    jboolean has_donation,
    jlong donation_qualified_amount
) {
    (void)cls;
    JLawIncomeDeductionInput input;
    memset(&input, 0, sizeof(input));
    if (!fill_income_deduction_input(
        env,
        &input,
        total_income_amount,
        year,
        month,
        day,
        has_spouse,
        spouse_total_income_amount,
        spouse_is_same_household,
        spouse_is_elderly,
        dependent_general_count,
        dependent_specific_count,
        dependent_elderly_cohabiting_count,
        dependent_elderly_other_count,
        social_insurance_premium_paid,
        has_medical,
        medical_expense_paid,
        medical_reimbursed_amount,
        has_life_insurance,
        life_new_general_paid_amount,
        life_new_individual_pension_paid_amount,
        life_new_care_medical_paid_amount,
        life_old_general_paid_amount,
        life_old_individual_pension_paid_amount,
        has_donation,
        donation_qualified_amount
    )) {
        return NULL;
    }

    JLawIncomeDeductionResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_income_deductions(&input, &result, error_buf, J_LAW_ERROR_BUF_LEN);
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    return build_income_deduction_result(env, &result);
}

static jobject build_income_tax_result(JNIEnv *env, const JLawIncomeTaxResult *result) {
    jobjectArray breakdown = build_income_tax_step_array(env, result->breakdown, result->breakdown_len);
    if (breakdown == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeTaxResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(JJJZ[Lio/github/kmoyashi/jlaw/IncomeTaxStep;)V"
    );
    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result->base_tax,
        (jlong)result->reconstruction_tax,
        (jlong)result->total_tax,
        result->reconstruction_tax_applied != 0 ? JNI_TRUE : JNI_FALSE,
        breakdown
    );
    if (breakdown != NULL) {
        (*env)->DeleteLocalRef(env, breakdown);
    }
    return output;
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcIncomeTaxAssessment(
    JNIEnv *env,
    jclass cls,
    jlong total_income_amount,
    jint year,
    jint month,
    jint day,
    jboolean has_spouse,
    jlong spouse_total_income_amount,
    jboolean spouse_is_same_household,
    jboolean spouse_is_elderly,
    jlong dependent_general_count,
    jlong dependent_specific_count,
    jlong dependent_elderly_cohabiting_count,
    jlong dependent_elderly_other_count,
    jlong social_insurance_premium_paid,
    jboolean has_medical,
    jlong medical_expense_paid,
    jlong medical_reimbursed_amount,
    jboolean has_life_insurance,
    jlong life_new_general_paid_amount,
    jlong life_new_individual_pension_paid_amount,
    jlong life_new_care_medical_paid_amount,
    jlong life_old_general_paid_amount,
    jlong life_old_individual_pension_paid_amount,
    jboolean has_donation,
    jlong donation_qualified_amount,
    jboolean apply_reconstruction_tax
) {
    (void)cls;
    JLawIncomeDeductionInput input;
    memset(&input, 0, sizeof(input));
    if (!fill_income_deduction_input(
        env,
        &input,
        total_income_amount,
        year,
        month,
        day,
        has_spouse,
        spouse_total_income_amount,
        spouse_is_same_household,
        spouse_is_elderly,
        dependent_general_count,
        dependent_specific_count,
        dependent_elderly_cohabiting_count,
        dependent_elderly_other_count,
        social_insurance_premium_paid,
        has_medical,
        medical_expense_paid,
        medical_reimbursed_amount,
        has_life_insurance,
        life_new_general_paid_amount,
        life_new_individual_pension_paid_amount,
        life_new_care_medical_paid_amount,
        life_old_general_paid_amount,
        life_old_individual_pension_paid_amount,
        has_donation,
        donation_qualified_amount
    )) {
        return NULL;
    }

    JLawIncomeTaxAssessmentResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_income_tax_assessment(
        &input,
        apply_reconstruction_tax == JNI_TRUE ? 1 : 0,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    JLawIncomeDeductionResult deduction_result;
    deduction_result.total_income_amount = result.total_income_amount;
    deduction_result.total_deductions = result.total_deductions;
    deduction_result.taxable_income_before_truncation = result.taxable_income_before_truncation;
    deduction_result.taxable_income = result.taxable_income;
    deduction_result.breakdown_len = result.deduction_breakdown_len;
    memcpy(deduction_result.breakdown, result.deduction_breakdown, sizeof(result.deduction_breakdown));

    JLawIncomeTaxResult tax_result;
    tax_result.base_tax = result.base_tax;
    tax_result.reconstruction_tax = result.reconstruction_tax;
    tax_result.total_tax = result.total_tax;
    tax_result.reconstruction_tax_applied = result.reconstruction_tax_applied;
    tax_result.breakdown_len = result.tax_breakdown_len;
    memcpy(tax_result.breakdown, result.tax_breakdown, sizeof(result.tax_breakdown));

    jobject deduction_object = build_income_deduction_result(env, &deduction_result);
    if (deduction_object == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jobject tax_object = build_income_tax_result(env, &tax_result);
    if (tax_object == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/IncomeTaxAssessmentResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(Lio/github/kmoyashi/jlaw/IncomeDeductionResult;Lio/github/kmoyashi/jlaw/IncomeTaxResult;)V"
    );
    jobject output = (*env)->NewObject(env, result_class, ctor, deduction_object, tax_object);
    if (deduction_object != NULL) {
        (*env)->DeleteLocalRef(env, deduction_object);
    }
    if (tax_object != NULL) {
        (*env)->DeleteLocalRef(env, tax_object);
    }
    return output;
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcConsumptionTax(
    JNIEnv *env,
    jclass cls,
    jlong amount,
    jint year,
    jint month,
    jint day,
    jboolean is_reduced_rate
) {
    (void)cls;
    uint64_t amount_u64;
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, amount, "amount", &amount_u64)
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)) {
        return NULL;
    }

    JLawConsumptionTaxResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_consumption_tax(
        amount_u64,
        year_u16,
        month_u8,
        day_u8,
        is_reduced_rate == JNI_TRUE ? 1 : 0,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/ConsumptionTaxResult");
    jmethodID ctor = (*env)->GetMethodID(env, result_class, "<init>", "(JJJJJZ)V");
    return (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result.tax_amount,
        (jlong)result.amount_with_tax,
        (jlong)result.amount_without_tax,
        (jlong)result.applied_rate_numer,
        (jlong)result.applied_rate_denom,
        result.is_reduced_rate != 0 ? JNI_TRUE : JNI_FALSE
    );
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcStampTax(
    JNIEnv *env,
    jclass cls,
    jint document_code,
    jboolean has_stated_amount,
    jlong stated_amount,
    jint year,
    jint month,
    jint day,
    jlong flags_bitset
) {
    (void)cls;
    uint64_t stated_amount_u64;
    uint64_t flags_bitset_u64;
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, stated_amount, "statedAmount", &stated_amount_u64)
        || !validate_u64(env, flags_bitset, "flagsBitset", &flags_bitset_u64)
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)) {
        return NULL;
    }

    JLawStampTaxResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_stamp_tax(
        (uint32_t)document_code,
        stated_amount_u64,
        has_stated_amount == JNI_TRUE ? 1 : 0,
        year_u16,
        month_u8,
        day_u8,
        flags_bitset_u64,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    jstring rule_label = new_string(env, result.rule_label);
    jstring applied_special_rule = new_string(env, result.applied_special_rule);
    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/StampTaxResult");
    jmethodID ctor = (*env)->GetMethodID(env, result_class, "<init>", "(JLjava/lang/String;Ljava/lang/String;)V");
    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result.tax_amount,
        rule_label,
        applied_special_rule
    );
    if (rule_label != NULL) {
        (*env)->DeleteLocalRef(env, rule_label);
    }
    if (applied_special_rule != NULL) {
        (*env)->DeleteLocalRef(env, applied_special_rule);
    }
    return output;
}

JNIEXPORT jobject JNICALL Java_io_github_kmoyashi_jlaw_internal_NativeBridge_calcWithholdingTax(
    JNIEnv *env,
    jclass cls,
    jlong payment_amount,
    jlong separated_consumption_tax_amount,
    jint year,
    jint month,
    jint day,
    jint category_code,
    jboolean is_submission_prize
) {
    (void)cls;
    uint64_t payment_amount_u64;
    uint64_t separated_consumption_tax_amount_u64;
    uint16_t year_u16;
    uint8_t month_u8;
    uint8_t day_u8;
    if (!validate_u64(env, payment_amount, "paymentAmount", &payment_amount_u64)
        || !validate_u64(
            env,
            separated_consumption_tax_amount,
            "separatedConsumptionTaxAmount",
            &separated_consumption_tax_amount_u64
        )
        || !validate_u16(env, year, "year", &year_u16)
        || !validate_u8(env, month, "month", &month_u8)
        || !validate_u8(env, day, "day", &day_u8)) {
        return NULL;
    }

    JLawWithholdingTaxResult result;
    char error_buf[J_LAW_ERROR_BUF_LEN] = {0};
    int status = j_law_calc_withholding_tax(
        payment_amount_u64,
        separated_consumption_tax_amount_u64,
        year_u16,
        month_u8,
        day_u8,
        (uint32_t)category_code,
        is_submission_prize == JNI_TRUE ? 1 : 0,
        &result,
        error_buf,
        J_LAW_ERROR_BUF_LEN
    );
    if (status != 0) {
        throw_jlaw_exception(env, error_buf);
        return NULL;
    }

    jobjectArray breakdown = build_breakdown_array(env, result.breakdown, result.breakdown_len);
    if (breakdown == NULL && (*env)->ExceptionCheck(env)) {
        return NULL;
    }

    jclass result_class = (*env)->FindClass(env, "io/github/kmoyashi/jlaw/WithholdingTaxResult");
    jmethodID ctor = (*env)->GetMethodID(
        env,
        result_class,
        "<init>",
        "(JJJJIZ[Lio/github/kmoyashi/jlaw/BreakdownStep;)V"
    );
    jobject output = (*env)->NewObject(
        env,
        result_class,
        ctor,
        (jlong)result.gross_payment_amount,
        (jlong)result.taxable_payment_amount,
        (jlong)result.tax_amount,
        (jlong)result.net_payment_amount,
        (jint)result.category,
        result.submission_prize_exempted != 0 ? JNI_TRUE : JNI_FALSE,
        breakdown
    );
    if (breakdown != NULL) {
        (*env)->DeleteLocalRef(env, breakdown);
    }
    return output;
}
