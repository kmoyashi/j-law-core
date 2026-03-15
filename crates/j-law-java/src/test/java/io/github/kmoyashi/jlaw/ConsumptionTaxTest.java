package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import static org.junit.jupiter.api.Assertions.assertEquals;

class ConsumptionTaxTest {
    @TestFactory
    Collection<DynamicTest> consumptionTaxFixtures() {
        JsonNode cases = FixtureLoader.load("consumption_tax.json").get("consumption_tax");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                ConsumptionTaxResult result = ConsumptionTax.calcConsumptionTax(
                    input.get("amount").longValue(),
                    LocalDate.parse(input.get("date").textValue()),
                    input.get("is_reduced_rate").booleanValue()
                );

                assertEquals(expected.get("tax_amount").longValue(), result.getTaxAmount());
                assertEquals(expected.get("amount_with_tax").longValue(), result.getAmountWithTax());
                assertEquals(expected.get("amount_without_tax").longValue(), result.getAmountWithoutTax());
                assertEquals(expected.get("applied_rate_numer").longValue(), result.getAppliedRateNumer());
                assertEquals(expected.get("applied_rate_denom").longValue(), result.getAppliedRateDenom());
                assertEquals(expected.get("is_reduced_rate").booleanValue(), result.isReducedRate());
            }));
        }
        return tests;
    }
}
