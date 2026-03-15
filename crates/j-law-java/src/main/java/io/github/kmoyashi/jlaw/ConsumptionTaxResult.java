package io.github.kmoyashi.jlaw;

/**
 * Result of the consumption tax calculation.
 */
public final class ConsumptionTaxResult {
    private final long taxAmount;
    private final long amountWithTax;
    private final long amountWithoutTax;
    private final long appliedRateNumer;
    private final long appliedRateDenom;
    private final boolean reducedRate;

    public ConsumptionTaxResult(
        long taxAmount,
        long amountWithTax,
        long amountWithoutTax,
        long appliedRateNumer,
        long appliedRateDenom,
        boolean reducedRate
    ) {
        this.taxAmount = taxAmount;
        this.amountWithTax = amountWithTax;
        this.amountWithoutTax = amountWithoutTax;
        this.appliedRateNumer = appliedRateNumer;
        this.appliedRateDenom = appliedRateDenom;
        this.reducedRate = reducedRate;
    }

    public long getTaxAmount() {
        return taxAmount;
    }

    public long getAmountWithTax() {
        return amountWithTax;
    }

    public long getAmountWithoutTax() {
        return amountWithoutTax;
    }

    public long getAppliedRateNumer() {
        return appliedRateNumer;
    }

    public long getAppliedRateDenom() {
        return appliedRateDenom;
    }

    public boolean isReducedRate() {
        return reducedRate;
    }
}
