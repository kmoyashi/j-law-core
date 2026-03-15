package io.github.kmoyashi.jlaw;

/**
 * One breakdown step returned by brokerage fee or withholding tax calculations.
 */
public final class BreakdownStep {
    private final String label;
    private final long baseAmount;
    private final long rateNumer;
    private final long rateDenom;
    private final long result;

    public BreakdownStep(String label, long baseAmount, long rateNumer, long rateDenom, long result) {
        this.label = label;
        this.baseAmount = baseAmount;
        this.rateNumer = rateNumer;
        this.rateDenom = rateDenom;
        this.result = result;
    }

    public String getLabel() {
        return label;
    }

    public long getBaseAmount() {
        return baseAmount;
    }

    public long getRateNumer() {
        return rateNumer;
    }

    public long getRateDenom() {
        return rateDenom;
    }

    public long getResult() {
        return result;
    }
}
