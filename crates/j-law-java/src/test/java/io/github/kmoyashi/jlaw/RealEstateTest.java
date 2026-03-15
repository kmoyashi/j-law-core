package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import static org.junit.jupiter.api.Assertions.assertEquals;

class RealEstateTest {
    @TestFactory
    Collection<DynamicTest> brokerageFeeFixtures() {
        JsonNode cases = FixtureLoader.load("real_estate.json").get("brokerage_fee");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                BrokerageFeeResult result = RealEstate.calcBrokerageFee(
                    input.get("price").longValue(),
                    LocalDate.parse(input.get("date").textValue()),
                    input.path("is_low_cost_vacant_house").booleanValue(),
                    input.path("is_seller").booleanValue()
                );

                if (expected.has("total_without_tax")) {
                    assertEquals(expected.get("total_without_tax").longValue(), result.getTotalWithoutTax());
                }
                if (expected.has("tax_amount")) {
                    assertEquals(expected.get("tax_amount").longValue(), result.getTaxAmount());
                }
                if (expected.has("total_with_tax")) {
                    assertEquals(expected.get("total_with_tax").longValue(), result.getTotalWithTax());
                }
                if (expected.has("low_cost_special_applied")) {
                    assertEquals(expected.get("low_cost_special_applied").booleanValue(), result.isLowCostSpecialApplied());
                }
                if (expected.has("breakdown_results")) {
                    List<Long> actual = new ArrayList<Long>();
                    for (BreakdownStep step : result.getBreakdown()) {
                        actual.add(Long.valueOf(step.getResult()));
                    }
                    assertEquals(expected.get("breakdown_results").size(), actual.size());
                    for (int i = 0; i < actual.size(); i++) {
                        assertEquals(expected.get("breakdown_results").get(i).longValue(), actual.get(i).longValue());
                    }
                }
            }));
        }
        return tests;
    }
}
