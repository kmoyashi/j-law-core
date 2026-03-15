package io.github.kmoyashi.jlaw;

/**
 * Combined result from income deductions through total income tax.
 */
public final class IncomeTaxAssessmentResult {
    private final IncomeDeductionResult deductions;
    private final IncomeTaxResult tax;

    public IncomeTaxAssessmentResult(IncomeDeductionResult deductions, IncomeTaxResult tax) {
        this.deductions = deductions;
        this.tax = tax;
    }

    public IncomeDeductionResult getDeductions() {
        return deductions;
    }

    public IncomeTaxResult getTax() {
        return tax;
    }
}
