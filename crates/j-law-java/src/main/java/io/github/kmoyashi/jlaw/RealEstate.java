package io.github.kmoyashi.jlaw;

import io.github.kmoyashi.jlaw.internal.NativeBridge;
import io.github.kmoyashi.jlaw.internal.Validation;
import java.time.LocalDate;

/**
 * Real estate brokerage fee calculations.
 */
public final class RealEstate {
    private RealEstate() {
    }

    /**
     * Calculates the maximum brokerage fee under the Real Estate Brokerage Act Article 46.
     *
     * <p>WARNING: Whether a property qualifies as a low-cost vacant house is outside this library's scope.
     * Callers must determine the facts before setting {@code isLowCostVacantHouse}.</p>
     *
     * <p>WARNING: Whether the transaction should be treated as a seller-side transaction is outside this
     * library's scope. Callers must determine the facts before setting {@code isSeller}.</p>
     */
    public static BrokerageFeeResult calcBrokerageFee(
        long price,
        LocalDate date,
        boolean isLowCostVacantHouse,
        boolean isSeller
    ) {
        Validation.requireNonNegative(price, "price");
        LocalDate safeDate = Validation.requireDate(date, "date");
        NativeBridge.ensureLoaded();
        return NativeBridge.calcBrokerageFee(
            price,
            safeDate.getYear(),
            safeDate.getMonthValue(),
            safeDate.getDayOfMonth(),
            isLowCostVacantHouse,
            isSeller
        );
    }
}
