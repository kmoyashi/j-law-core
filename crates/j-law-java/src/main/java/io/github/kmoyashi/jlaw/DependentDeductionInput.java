package io.github.kmoyashi.jlaw;

/**
 * Input facts for dependent deductions.
 */
public final class DependentDeductionInput {
    private final long generalCount;
    private final long specificCount;
    private final long elderlyCohabitingCount;
    private final long elderlyOtherCount;

    public DependentDeductionInput(
        long generalCount,
        long specificCount,
        long elderlyCohabitingCount,
        long elderlyOtherCount
    ) {
        this.generalCount = generalCount;
        this.specificCount = specificCount;
        this.elderlyCohabitingCount = elderlyCohabitingCount;
        this.elderlyOtherCount = elderlyOtherCount;
    }

    public long getGeneralCount() {
        return generalCount;
    }

    public long getSpecificCount() {
        return specificCount;
    }

    public long getElderlyCohabitingCount() {
        return elderlyCohabitingCount;
    }

    public long getElderlyOtherCount() {
        return elderlyOtherCount;
    }
}
