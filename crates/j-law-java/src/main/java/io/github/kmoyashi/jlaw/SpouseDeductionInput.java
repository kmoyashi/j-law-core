package io.github.kmoyashi.jlaw;

/**
 * Input facts for the spouse deduction.
 */
public final class SpouseDeductionInput {
    private final long spouseTotalIncomeAmount;
    private final boolean sameHousehold;
    private final boolean elderly;

    public SpouseDeductionInput(long spouseTotalIncomeAmount, boolean sameHousehold, boolean elderly) {
        this.spouseTotalIncomeAmount = spouseTotalIncomeAmount;
        this.sameHousehold = sameHousehold;
        this.elderly = elderly;
    }

    public long getSpouseTotalIncomeAmount() {
        return spouseTotalIncomeAmount;
    }

    public boolean isSameHousehold() {
        return sameHousehold;
    }

    public boolean isElderly() {
        return elderly;
    }
}
