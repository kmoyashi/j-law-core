package io.github.kmoyashi.jlaw;

/**
 * Input facts for the donation deduction.
 */
public final class DonationDeductionInput {
    private final long qualifiedDonationAmount;

    public DonationDeductionInput(long qualifiedDonationAmount) {
        this.qualifiedDonationAmount = qualifiedDonationAmount;
    }

    public long getQualifiedDonationAmount() {
        return qualifiedDonationAmount;
    }
}
