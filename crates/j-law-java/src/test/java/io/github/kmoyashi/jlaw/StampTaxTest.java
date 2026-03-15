package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.EnumSet;
import java.util.List;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class StampTaxTest {
    @TestFactory
    Collection<DynamicTest> stampTaxFixtures() {
        JsonNode cases = FixtureLoader.load("stamp_tax.json").get("stamp_tax");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                EnumSet<StampTaxFlag> flags = EnumSet.noneOf(StampTaxFlag.class);
                for (JsonNode flag : input.get("flags")) {
                    flags.add(FixtureLoader.stampTaxFlag(flag.textValue()));
                }

                Long statedAmount = input.get("stated_amount").isNull() ? null : Long.valueOf(input.get("stated_amount").longValue());

                StampTaxResult result = StampTax.calcStampTax(
                    FixtureLoader.stampTaxDocumentCode(input.get("document_code").textValue()),
                    statedAmount,
                    LocalDate.parse(input.get("date").textValue()),
                    flags
                );

                assertEquals(expected.get("tax_amount").longValue(), result.getTaxAmount());
                assertEquals(expected.get("rule_label").textValue(), result.getRuleLabel());
                if (expected.get("applied_special_rule").isNull()) {
                    assertNull(result.getAppliedSpecialRule());
                } else {
                    assertEquals(expected.get("applied_special_rule").textValue(), result.getAppliedSpecialRule());
                }
            }));
        }
        return tests;
    }
}
