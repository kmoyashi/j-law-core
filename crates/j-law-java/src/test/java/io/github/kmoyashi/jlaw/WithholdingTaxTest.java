package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import static org.junit.jupiter.api.Assertions.assertEquals;

class WithholdingTaxTest {
    @TestFactory
    Collection<DynamicTest> withholdingTaxFixtures() {
        JsonNode cases = FixtureLoader.load("withholding_tax.json").get("withholding_tax");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                WithholdingTaxResult result = WithholdingTax.calcWithholdingTax(
                    input.get("payment_amount").longValue(),
                    input.get("separated_consumption_tax_amount").longValue(),
                    LocalDate.parse(input.get("date").textValue()),
                    FixtureLoader.withholdingTaxCategory(input.get("category").textValue()),
                    input.get("is_submission_prize").booleanValue()
                );

                assertEquals(expected.get("taxable_payment_amount").longValue(), result.getTaxablePaymentAmount());
                assertEquals(expected.get("tax_amount").longValue(), result.getTaxAmount());
                assertEquals(expected.get("net_payment_amount").longValue(), result.getNetPaymentAmount());
                assertEquals(expected.get("submission_prize_exempted").booleanValue(), result.isSubmissionPrizeExempted());
            }));
        }
        return tests;
    }
}
