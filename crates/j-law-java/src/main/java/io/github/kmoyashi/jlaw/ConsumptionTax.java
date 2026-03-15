package io.github.kmoyashi.jlaw;

import io.github.kmoyashi.jlaw.internal.NativeBridge;
import io.github.kmoyashi.jlaw.internal.Validation;
import java.time.LocalDate;

/**
 * Consumption tax calculations.
 */
public final class ConsumptionTax {
    private ConsumptionTax() {
    }

    /**
     * Calculates consumption tax for the given amount and date.
     *
     * <p>WARNING: Whether the reduced-rate regime applies is outside this library's scope. Callers must
     * determine the facts before setting {@code isReducedRate}.</p>
     */
    public static ConsumptionTaxResult calcConsumptionTax(long amount, LocalDate date, boolean isReducedRate) {
        Validation.requireNonNegative(amount, "amount");
        LocalDate safeDate = Validation.requireDate(date, "date");
        NativeBridge.ensureLoaded();
        return NativeBridge.calcConsumptionTax(
            amount,
            safeDate.getYear(),
            safeDate.getMonthValue(),
            safeDate.getDayOfMonth(),
            isReducedRate
        );
    }
}
