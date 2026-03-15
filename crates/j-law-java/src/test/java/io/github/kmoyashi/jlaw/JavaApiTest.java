package io.github.kmoyashi.jlaw;

import java.time.LocalDate;
import java.util.Collections;
import java.util.EnumSet;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class JavaApiTest {
    @Test
    void rejectsNegativeAmounts() {
        assertThrows(IllegalArgumentException.class, () -> RealEstate.calcBrokerageFee(-1L, LocalDate.of(2024, 8, 1), false, false));
        assertThrows(
            IllegalArgumentException.class,
            () -> WithholdingTax.calcWithholdingTax(1L, -1L, LocalDate.of(2026, 1, 1), WithholdingTaxCategory.MANUSCRIPT_AND_LECTURE, false)
        );
    }

    @Test
    void rejectsDateOutsideU16YearRange() {
        assertThrows(
            IllegalArgumentException.class,
            () -> ConsumptionTax.calcConsumptionTax(100L, LocalDate.of(100000, 1, 1), false)
        );
    }

    @Test
    void breakdownIsImmutable() {
        BrokerageFeeResult result = RealEstate.calcBrokerageFee(5_000_000L, LocalDate.of(2024, 8, 1), false, false);
        assertThrows(UnsupportedOperationException.class, () -> result.getBreakdown().add(new BreakdownStep("x", 0L, 0L, 1L, 0L)));
    }

    @Test
    void stampTaxAllowsNullStatedAmountAndNullFlags() {
        StampTaxResult result = StampTax.calcStampTax(
            StampTaxDocumentCode.ARTICLE1_REAL_ESTATE_TRANSFER,
            null,
            LocalDate.of(2027, 4, 1),
            null
        );
        assertEquals(200L, result.getTaxAmount());
        assertEquals("契約金額の記載のないもの", result.getRuleLabel());
    }

    @Test
    void enumCodeConversionRejectsUnknownValue() {
        assertThrows(IllegalArgumentException.class, () -> WithholdingTaxCategory.fromCode(999));
    }

    @Test
    void concurrentCallsShareTheSameLoadedNativeLibrary() throws ExecutionException, InterruptedException {
        ExecutorService executor = Executors.newFixedThreadPool(4);
        try {
            Callable<Long> task = new Callable<Long>() {
                @Override
                public Long call() {
                    return Long.valueOf(
                        RealEstate.calcBrokerageFee(5_000_000L, LocalDate.of(2024, 8, 1), false, false).getTotalWithTax()
                    );
                }
            };

            Future<Long> first = executor.submit(task);
            Future<Long> second = executor.submit(task);
            Future<Long> third = executor.submit(task);
            Future<Long> fourth = executor.submit(task);

            assertEquals(Long.valueOf(231000L), first.get());
            assertEquals(Long.valueOf(231000L), second.get());
            assertEquals(Long.valueOf(231000L), third.get());
            assertEquals(Long.valueOf(231000L), fourth.get());
        } finally {
            executor.shutdownNow();
        }
    }

    @Test
    void builderHandlesOptionalInputs() {
        IncomeDeductionInput input = new IncomeDeductionInput.Builder(6_000_000L, LocalDate.of(2024, 1, 1))
            .socialInsurancePremiumPaid(150_000L)
            .medical(new MedicalDeductionInput(500_000L, 50_000L))
            .lifeInsurance(new LifeInsuranceDeductionInput(100_000L, 60_000L, 80_000L, 0L, 0L))
            .donation(new DonationDeductionInput(500_000L))
            .build();

        IncomeDeductionResult result = IncomeTax.calcIncomeDeductions(input);
        assertEquals(1_593_000L, result.getTotalDeductions());
    }
}
