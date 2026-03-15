package io.github.kmoyashi.jlaw;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

/**
 * Calculation result for brokerage fees under the Real Estate Brokerage Act Article 46.
 */
public final class BrokerageFeeResult {
    private final long totalWithoutTax;
    private final long totalWithTax;
    private final long taxAmount;
    private final boolean lowCostSpecialApplied;
    private final List<BreakdownStep> breakdown;

    public BrokerageFeeResult(
        long totalWithoutTax,
        long totalWithTax,
        long taxAmount,
        boolean lowCostSpecialApplied,
        BreakdownStep[] breakdown
    ) {
        this.totalWithoutTax = totalWithoutTax;
        this.totalWithTax = totalWithTax;
        this.taxAmount = taxAmount;
        this.lowCostSpecialApplied = lowCostSpecialApplied;
        this.breakdown = immutableList(breakdown);
    }

    public long getTotalWithoutTax() {
        return totalWithoutTax;
    }

    public long getTotalWithTax() {
        return totalWithTax;
    }

    public long getTaxAmount() {
        return taxAmount;
    }

    public boolean isLowCostSpecialApplied() {
        return lowCostSpecialApplied;
    }

    public List<BreakdownStep> getBreakdown() {
        return breakdown;
    }

    private static List<BreakdownStep> immutableList(BreakdownStep[] values) {
        BreakdownStep[] copy = values == null ? new BreakdownStep[0] : Arrays.copyOf(values, values.length);
        return Collections.unmodifiableList(new ArrayList<BreakdownStep>(Arrays.asList(copy)));
    }
}
