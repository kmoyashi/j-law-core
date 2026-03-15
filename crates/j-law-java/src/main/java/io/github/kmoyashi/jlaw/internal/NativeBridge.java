package io.github.kmoyashi.jlaw.internal;

import io.github.kmoyashi.jlaw.BrokerageFeeResult;
import io.github.kmoyashi.jlaw.ConsumptionTaxResult;
import io.github.kmoyashi.jlaw.IncomeDeductionResult;
import io.github.kmoyashi.jlaw.IncomeTaxAssessmentResult;
import io.github.kmoyashi.jlaw.IncomeTaxResult;
import io.github.kmoyashi.jlaw.JLawException;
import io.github.kmoyashi.jlaw.StampTaxResult;
import io.github.kmoyashi.jlaw.WithholdingTaxResult;

public final class NativeBridge {
    private static final int EXPECTED_FFI_VERSION = 4;

    static {
        NativeLoader.load();
        int actual = ffiVersion();
        if (actual != EXPECTED_FFI_VERSION) {
            throw new ExceptionInInitializerError(
                new JLawException("j-law-c-ffi version mismatch: expected " + EXPECTED_FFI_VERSION + ", got " + actual)
            );
        }
    }

    private NativeBridge() {
    }

    public static void ensureLoaded() {
    }

    public static native int ffiVersion();

    public static native BrokerageFeeResult calcBrokerageFee(
        long price,
        int year,
        int month,
        int day,
        boolean isLowCostVacantHouse,
        boolean isSeller
    );

    public static native IncomeTaxResult calcIncomeTax(
        long taxableIncome,
        int year,
        int month,
        int day,
        boolean applyReconstructionTax
    );

    public static native IncomeDeductionResult calcIncomeDeductions(
        long totalIncomeAmount,
        int year,
        int month,
        int day,
        boolean hasSpouse,
        long spouseTotalIncomeAmount,
        boolean spouseIsSameHousehold,
        boolean spouseIsElderly,
        long dependentGeneralCount,
        long dependentSpecificCount,
        long dependentElderlyCohabitingCount,
        long dependentElderlyOtherCount,
        long socialInsurancePremiumPaid,
        boolean hasMedical,
        long medicalExpensePaid,
        long medicalReimbursedAmount,
        boolean hasLifeInsurance,
        long lifeNewGeneralPaidAmount,
        long lifeNewIndividualPensionPaidAmount,
        long lifeNewCareMedicalPaidAmount,
        long lifeOldGeneralPaidAmount,
        long lifeOldIndividualPensionPaidAmount,
        boolean hasDonation,
        long donationQualifiedAmount
    );

    public static native IncomeTaxAssessmentResult calcIncomeTaxAssessment(
        long totalIncomeAmount,
        int year,
        int month,
        int day,
        boolean hasSpouse,
        long spouseTotalIncomeAmount,
        boolean spouseIsSameHousehold,
        boolean spouseIsElderly,
        long dependentGeneralCount,
        long dependentSpecificCount,
        long dependentElderlyCohabitingCount,
        long dependentElderlyOtherCount,
        long socialInsurancePremiumPaid,
        boolean hasMedical,
        long medicalExpensePaid,
        long medicalReimbursedAmount,
        boolean hasLifeInsurance,
        long lifeNewGeneralPaidAmount,
        long lifeNewIndividualPensionPaidAmount,
        long lifeNewCareMedicalPaidAmount,
        long lifeOldGeneralPaidAmount,
        long lifeOldIndividualPensionPaidAmount,
        boolean hasDonation,
        long donationQualifiedAmount,
        boolean applyReconstructionTax
    );

    public static native ConsumptionTaxResult calcConsumptionTax(
        long amount,
        int year,
        int month,
        int day,
        boolean isReducedRate
    );

    public static native StampTaxResult calcStampTax(
        int documentCode,
        boolean hasStatedAmount,
        long statedAmount,
        int year,
        int month,
        int day,
        long flagsBitset
    );

    public static native WithholdingTaxResult calcWithholdingTax(
        long paymentAmount,
        long separatedConsumptionTaxAmount,
        int year,
        int month,
        int day,
        int categoryCode,
        boolean isSubmissionPrize
    );
}
