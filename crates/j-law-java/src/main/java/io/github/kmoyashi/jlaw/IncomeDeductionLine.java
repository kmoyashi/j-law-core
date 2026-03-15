package io.github.kmoyashi.jlaw;

/**
 * One line in the income deduction breakdown.
 */
public final class IncomeDeductionLine {
    private final int kind;
    private final String label;
    private final long amount;

    public IncomeDeductionLine(int kind, String label, long amount) {
        this.kind = kind;
        this.label = label;
        this.amount = amount;
    }

    public int getKind() {
        return kind;
    }

    public String getLabel() {
        return label;
    }

    public long getAmount() {
        return amount;
    }
}
