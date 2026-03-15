package io.github.kmoyashi.jlaw;

/**
 * Input facts for the medical deduction.
 */
public final class MedicalDeductionInput {
    private final long medicalExpensePaid;
    private final long reimbursedAmount;

    public MedicalDeductionInput(long medicalExpensePaid, long reimbursedAmount) {
        this.medicalExpensePaid = medicalExpensePaid;
        this.reimbursedAmount = reimbursedAmount;
    }

    public long getMedicalExpensePaid() {
        return medicalExpensePaid;
    }

    public long getReimbursedAmount() {
        return reimbursedAmount;
    }
}
