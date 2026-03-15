package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import static org.junit.jupiter.api.Assertions.assertEquals;

class IncomeTaxTest {
    @TestFactory
    Collection<DynamicTest> incomeTaxFixtures() {
        JsonNode cases = FixtureLoader.load("income_tax.json").get("income_tax");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                IncomeTaxResult result = IncomeTax.calcIncomeTax(
                    input.get("taxable_income").longValue(),
                    LocalDate.parse(input.get("date").textValue()),
                    input.get("apply_reconstruction_tax").booleanValue()
                );

                assertEquals(expected.get("base_tax").longValue(), result.getBaseTax());
                assertEquals(expected.get("reconstruction_tax").longValue(), result.getReconstructionTax());
                assertEquals(expected.get("total_tax").longValue(), result.getTotalTax());
                assertEquals(expected.get("reconstruction_tax_applied").booleanValue(), result.isReconstructionTaxApplied());
            }));
        }
        return tests;
    }

    @TestFactory
    Collection<DynamicTest> incomeDeductionFixtures() {
        JsonNode cases = FixtureLoader.load("income_tax_deductions.json").get("income_tax_deductions");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                IncomeDeductionResult result = IncomeTax.calcIncomeDeductions(
                    FixtureLoader.incomeDeductionInput(caseNode.get("input"))
                );
                JsonNode expected = caseNode.get("expected");

                assertEquals(expected.get("total_income_amount").longValue(), result.getTotalIncomeAmount());
                assertEquals(expected.get("total_deductions").longValue(), result.getTotalDeductions());
                assertEquals(
                    expected.get("taxable_income_before_truncation").longValue(),
                    result.getTaxableIncomeBeforeTruncation()
                );
                assertEquals(expected.get("taxable_income").longValue(), result.getTaxableIncome());
            }));
        }
        return tests;
    }

    @TestFactory
    Collection<DynamicTest> incomeTaxAssessmentFixtures() {
        JsonNode cases = FixtureLoader.load("income_tax_deductions.json").get("income_tax_assessment");
        List<DynamicTest> tests = new ArrayList<DynamicTest>();
        for (JsonNode caseNode : cases) {
            tests.add(DynamicTest.dynamicTest(caseNode.get("id").textValue(), () -> {
                JsonNode input = caseNode.get("input");
                JsonNode expected = caseNode.get("expected");

                IncomeTaxAssessmentResult result = IncomeTax.calcIncomeTaxAssessment(
                    FixtureLoader.incomeDeductionInput(input),
                    input.get("apply_reconstruction_tax").booleanValue()
                );

                assertEquals(expected.get("taxable_income").longValue(), result.getDeductions().getTaxableIncome());
                assertEquals(expected.get("base_tax").longValue(), result.getTax().getBaseTax());
                assertEquals(expected.get("reconstruction_tax").longValue(), result.getTax().getReconstructionTax());
                assertEquals(expected.get("total_tax").longValue(), result.getTax().getTotalTax());
            }));
        }
        return tests;
    }
}
