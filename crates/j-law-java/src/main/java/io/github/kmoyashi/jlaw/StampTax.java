package io.github.kmoyashi.jlaw;

import io.github.kmoyashi.jlaw.internal.NativeBridge;
import io.github.kmoyashi.jlaw.internal.Validation;
import java.time.LocalDate;
import java.util.EnumSet;
import java.util.Objects;

/**
 * Stamp tax calculations.
 */
public final class StampTax {
    private StampTax() {
    }

    /**
     * Calculates stamp tax for the given document.
     */
    public static StampTaxResult calcStampTax(
        StampTaxDocumentCode documentCode,
        Long statedAmount,
        LocalDate date,
        EnumSet<StampTaxFlag> flags
    ) {
        Objects.requireNonNull(documentCode, "documentCode must not be null");
        LocalDate safeDate = Validation.requireDate(date, "date");
        long safeStatedAmount = 0L;
        boolean hasStatedAmount = statedAmount != null;
        if (hasStatedAmount) {
            safeStatedAmount = Validation.requireNonNegative(statedAmount.longValue(), "statedAmount");
        }

        long bitset = 0L;
        if (flags != null) {
            for (StampTaxFlag flag : flags) {
                if (flag == null) {
                    throw new IllegalArgumentException("flags must not contain null");
                }
                bitset |= flag.getBitMask();
            }
        }

        NativeBridge.ensureLoaded();
        return NativeBridge.calcStampTax(
            documentCode.getCode(),
            hasStatedAmount,
            safeStatedAmount,
            safeDate.getYear(),
            safeDate.getMonthValue(),
            safeDate.getDayOfMonth(),
            bitset
        );
    }
}
