package io.github.kmoyashi.jlaw;

import io.github.kmoyashi.jlaw.internal.NativeBridge;
import io.github.kmoyashi.jlaw.internal.Validation;
import java.time.LocalDate;

/**
 * Income tax and income deduction calculations.
 */
public final class IncomeTax {
    private IncomeTax() {
    }

    /**
     * Calculates income tax from taxable income.
     */
    public static IncomeTaxResult calcIncomeTax(long taxableIncome, LocalDate date, boolean applyReconstructionTax) {
        Validation.requireNonNegative(taxableIncome, "taxableIncome");
        LocalDate safeDate = Validation.requireDate(date, "date");
        NativeBridge.ensureLoaded();
        return NativeBridge.calcIncomeTax(
            taxableIncome,
            safeDate.getYear(),
            safeDate.getMonthValue(),
            safeDate.getDayOfMonth(),
            applyReconstructionTax
        );
    }

    /**
     * Calculates deductions and taxable income.
     */
    public static IncomeDeductionResult calcIncomeDeductions(IncomeDeductionInput input) {
        Validation.validateIncomeDeductionInput(input);
        return NativeBridge.calcIncomeDeductions(
            input.getTotalIncomeAmount(),
            input.getDate().getYear(),
            input.getDate().getMonthValue(),
            input.getDate().getDayOfMonth(),
            input.getSpouse() != null,
            input.getSpouse() == null ? 0L : input.getSpouse().getSpouseTotalIncomeAmount(),
            input.getSpouse() != null && input.getSpouse().isSameHousehold(),
            input.getSpouse() != null && input.getSpouse().isElderly(),
            input.getDependent().getGeneralCount(),
            input.getDependent().getSpecificCount(),
            input.getDependent().getElderlyCohabitingCount(),
            input.getDependent().getElderlyOtherCount(),
            input.getSocialInsurancePremiumPaid(),
            input.getMedical() != null,
            input.getMedical() == null ? 0L : input.getMedical().getMedicalExpensePaid(),
            input.getMedical() == null ? 0L : input.getMedical().getReimbursedAmount(),
            input.getLifeInsurance() != null,
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewGeneralPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewIndividualPensionPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewCareMedicalPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getOldGeneralPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getOldIndividualPensionPaidAmount(),
            input.getDonation() != null,
            input.getDonation() == null ? 0L : input.getDonation().getQualifiedDonationAmount()
        );
    }

    /**
     * Calculates deductions through total income tax in one call.
     */
    public static IncomeTaxAssessmentResult calcIncomeTaxAssessment(
        IncomeDeductionInput input,
        boolean applyReconstructionTax
    ) {
        Validation.validateIncomeDeductionInput(input);
        return NativeBridge.calcIncomeTaxAssessment(
            input.getTotalIncomeAmount(),
            input.getDate().getYear(),
            input.getDate().getMonthValue(),
            input.getDate().getDayOfMonth(),
            input.getSpouse() != null,
            input.getSpouse() == null ? 0L : input.getSpouse().getSpouseTotalIncomeAmount(),
            input.getSpouse() != null && input.getSpouse().isSameHousehold(),
            input.getSpouse() != null && input.getSpouse().isElderly(),
            input.getDependent().getGeneralCount(),
            input.getDependent().getSpecificCount(),
            input.getDependent().getElderlyCohabitingCount(),
            input.getDependent().getElderlyOtherCount(),
            input.getSocialInsurancePremiumPaid(),
            input.getMedical() != null,
            input.getMedical() == null ? 0L : input.getMedical().getMedicalExpensePaid(),
            input.getMedical() == null ? 0L : input.getMedical().getReimbursedAmount(),
            input.getLifeInsurance() != null,
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewGeneralPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewIndividualPensionPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getNewCareMedicalPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getOldGeneralPaidAmount(),
            input.getLifeInsurance() == null ? 0L : input.getLifeInsurance().getOldIndividualPensionPaidAmount(),
            input.getDonation() != null,
            input.getDonation() == null ? 0L : input.getDonation().getQualifiedDonationAmount(),
            applyReconstructionTax
        );
    }
}
