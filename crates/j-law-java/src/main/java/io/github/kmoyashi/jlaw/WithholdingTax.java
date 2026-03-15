package io.github.kmoyashi.jlaw;

import io.github.kmoyashi.jlaw.internal.NativeBridge;
import io.github.kmoyashi.jlaw.internal.Validation;
import java.time.LocalDate;
import java.util.Objects;

/**
 * Withholding tax calculations.
 */
public final class WithholdingTax {
    private WithholdingTax() {
    }

    /**
     * Calculates withholding tax for reports, fees, and similar payments.
     *
     * <p>WARNING: Whether a payment qualifies as a submission prize is outside this library's scope.
     * Callers must determine the facts before setting {@code isSubmissionPrize}.</p>
     */
    public static WithholdingTaxResult calcWithholdingTax(
        long paymentAmount,
        long separatedConsumptionTaxAmount,
        LocalDate date,
        WithholdingTaxCategory category,
        boolean isSubmissionPrize
    ) {
        Validation.requireNonNegative(paymentAmount, "paymentAmount");
        Validation.requireNonNegative(separatedConsumptionTaxAmount, "separatedConsumptionTaxAmount");
        LocalDate safeDate = Validation.requireDate(date, "date");
        Objects.requireNonNull(category, "category must not be null");
        NativeBridge.ensureLoaded();
        return NativeBridge.calcWithholdingTax(
            paymentAmount,
            separatedConsumptionTaxAmount,
            safeDate.getYear(),
            safeDate.getMonthValue(),
            safeDate.getDayOfMonth(),
            category.getCode(),
            isSubmissionPrize
        );
    }
}
