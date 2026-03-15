package io.github.kmoyashi.jlaw;

/**
 * Result of the stamp tax calculation.
 */
public final class StampTaxResult {
    private final long taxAmount;
    private final String ruleLabel;
    private final String appliedSpecialRule;

    public StampTaxResult(long taxAmount, String ruleLabel, String appliedSpecialRule) {
        this.taxAmount = taxAmount;
        this.ruleLabel = ruleLabel;
        this.appliedSpecialRule = appliedSpecialRule;
    }

    public long getTaxAmount() {
        return taxAmount;
    }

    public String getRuleLabel() {
        return ruleLabel;
    }

    public String getAppliedSpecialRule() {
        return appliedSpecialRule;
    }
}
