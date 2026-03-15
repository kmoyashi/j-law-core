package io.github.kmoyashi.jlaw;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

/**
 * Result of a withholding tax calculation.
 */
public final class WithholdingTaxResult {
    private final long grossPaymentAmount;
    private final long taxablePaymentAmount;
    private final long taxAmount;
    private final long netPaymentAmount;
    private final WithholdingTaxCategory category;
    private final boolean submissionPrizeExempted;
    private final List<BreakdownStep> breakdown;

    public WithholdingTaxResult(
        long grossPaymentAmount,
        long taxablePaymentAmount,
        long taxAmount,
        long netPaymentAmount,
        int categoryCode,
        boolean submissionPrizeExempted,
        BreakdownStep[] breakdown
    ) {
        this.grossPaymentAmount = grossPaymentAmount;
        this.taxablePaymentAmount = taxablePaymentAmount;
        this.taxAmount = taxAmount;
        this.netPaymentAmount = netPaymentAmount;
        this.category = WithholdingTaxCategory.fromCode(categoryCode);
        this.submissionPrizeExempted = submissionPrizeExempted;
        this.breakdown = immutableList(breakdown);
    }

    public long getGrossPaymentAmount() {
        return grossPaymentAmount;
    }

    public long getTaxablePaymentAmount() {
        return taxablePaymentAmount;
    }

    public long getTaxAmount() {
        return taxAmount;
    }

    public long getNetPaymentAmount() {
        return netPaymentAmount;
    }

    public WithholdingTaxCategory getCategory() {
        return category;
    }

    public boolean isSubmissionPrizeExempted() {
        return submissionPrizeExempted;
    }

    public List<BreakdownStep> getBreakdown() {
        return breakdown;
    }

    private static List<BreakdownStep> immutableList(BreakdownStep[] values) {
        BreakdownStep[] copy = values == null ? new BreakdownStep[0] : Arrays.copyOf(values, values.length);
        return Collections.unmodifiableList(new ArrayList<BreakdownStep>(Arrays.asList(copy)));
    }
}
