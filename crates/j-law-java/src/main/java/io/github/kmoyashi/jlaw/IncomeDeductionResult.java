package io.github.kmoyashi.jlaw;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

/**
 * Result of the income deduction calculation.
 */
public final class IncomeDeductionResult {
    private final long totalIncomeAmount;
    private final long totalDeductions;
    private final long taxableIncomeBeforeTruncation;
    private final long taxableIncome;
    private final List<IncomeDeductionLine> breakdown;

    public IncomeDeductionResult(
        long totalIncomeAmount,
        long totalDeductions,
        long taxableIncomeBeforeTruncation,
        long taxableIncome,
        IncomeDeductionLine[] breakdown
    ) {
        this.totalIncomeAmount = totalIncomeAmount;
        this.totalDeductions = totalDeductions;
        this.taxableIncomeBeforeTruncation = taxableIncomeBeforeTruncation;
        this.taxableIncome = taxableIncome;
        this.breakdown = immutableList(breakdown);
    }

    public long getTotalIncomeAmount() {
        return totalIncomeAmount;
    }

    public long getTotalDeductions() {
        return totalDeductions;
    }

    public long getTaxableIncomeBeforeTruncation() {
        return taxableIncomeBeforeTruncation;
    }

    public long getTaxableIncome() {
        return taxableIncome;
    }

    public List<IncomeDeductionLine> getBreakdown() {
        return breakdown;
    }

    private static List<IncomeDeductionLine> immutableList(IncomeDeductionLine[] values) {
        IncomeDeductionLine[] copy = values == null ? new IncomeDeductionLine[0] : Arrays.copyOf(values, values.length);
        return Collections.unmodifiableList(new ArrayList<IncomeDeductionLine>(Arrays.asList(copy)));
    }
}
