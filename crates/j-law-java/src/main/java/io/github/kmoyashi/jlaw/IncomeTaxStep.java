package io.github.kmoyashi.jlaw;

/**
 * One quick-table step used in the income tax result.
 */
public final class IncomeTaxStep {
    private final String label;
    private final long taxableIncome;
    private final long rateNumer;
    private final long rateDenom;
    private final long deduction;
    private final long result;

    public IncomeTaxStep(
        String label,
        long taxableIncome,
        long rateNumer,
        long rateDenom,
        long deduction,
        long result
    ) {
        this.label = label;
        this.taxableIncome = taxableIncome;
        this.rateNumer = rateNumer;
        this.rateDenom = rateDenom;
        this.deduction = deduction;
        this.result = result;
    }

    public String getLabel() {
        return label;
    }

    public long getTaxableIncome() {
        return taxableIncome;
    }

    public long getRateNumer() {
        return rateNumer;
    }

    public long getRateDenom() {
        return rateDenom;
    }

    public long getDeduction() {
        return deduction;
    }

    public long getResult() {
        return result;
    }
}
