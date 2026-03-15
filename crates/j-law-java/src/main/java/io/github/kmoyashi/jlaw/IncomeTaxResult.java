package io.github.kmoyashi.jlaw;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

/**
 * Result of the income tax quick-table calculation.
 */
public final class IncomeTaxResult {
    private final long baseTax;
    private final long reconstructionTax;
    private final long totalTax;
    private final boolean reconstructionTaxApplied;
    private final List<IncomeTaxStep> breakdown;

    public IncomeTaxResult(
        long baseTax,
        long reconstructionTax,
        long totalTax,
        boolean reconstructionTaxApplied,
        IncomeTaxStep[] breakdown
    ) {
        this.baseTax = baseTax;
        this.reconstructionTax = reconstructionTax;
        this.totalTax = totalTax;
        this.reconstructionTaxApplied = reconstructionTaxApplied;
        this.breakdown = immutableList(breakdown);
    }

    public long getBaseTax() {
        return baseTax;
    }

    public long getReconstructionTax() {
        return reconstructionTax;
    }

    public long getTotalTax() {
        return totalTax;
    }

    public boolean isReconstructionTaxApplied() {
        return reconstructionTaxApplied;
    }

    public List<IncomeTaxStep> getBreakdown() {
        return breakdown;
    }

    private static List<IncomeTaxStep> immutableList(IncomeTaxStep[] values) {
        IncomeTaxStep[] copy = values == null ? new IncomeTaxStep[0] : Arrays.copyOf(values, values.length);
        return Collections.unmodifiableList(new ArrayList<IncomeTaxStep>(Arrays.asList(copy)));
    }
}
