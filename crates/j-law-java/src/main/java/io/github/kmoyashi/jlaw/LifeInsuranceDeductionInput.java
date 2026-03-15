package io.github.kmoyashi.jlaw;

/**
 * Input facts for the life insurance deduction.
 */
public final class LifeInsuranceDeductionInput {
    private final long newGeneralPaidAmount;
    private final long newIndividualPensionPaidAmount;
    private final long newCareMedicalPaidAmount;
    private final long oldGeneralPaidAmount;
    private final long oldIndividualPensionPaidAmount;

    public LifeInsuranceDeductionInput(
        long newGeneralPaidAmount,
        long newIndividualPensionPaidAmount,
        long newCareMedicalPaidAmount,
        long oldGeneralPaidAmount,
        long oldIndividualPensionPaidAmount
    ) {
        this.newGeneralPaidAmount = newGeneralPaidAmount;
        this.newIndividualPensionPaidAmount = newIndividualPensionPaidAmount;
        this.newCareMedicalPaidAmount = newCareMedicalPaidAmount;
        this.oldGeneralPaidAmount = oldGeneralPaidAmount;
        this.oldIndividualPensionPaidAmount = oldIndividualPensionPaidAmount;
    }

    public long getNewGeneralPaidAmount() {
        return newGeneralPaidAmount;
    }

    public long getNewIndividualPensionPaidAmount() {
        return newIndividualPensionPaidAmount;
    }

    public long getNewCareMedicalPaidAmount() {
        return newCareMedicalPaidAmount;
    }

    public long getOldGeneralPaidAmount() {
        return oldGeneralPaidAmount;
    }

    public long getOldIndividualPensionPaidAmount() {
        return oldIndividualPensionPaidAmount;
    }
}
