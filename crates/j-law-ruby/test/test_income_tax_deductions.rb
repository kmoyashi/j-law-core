# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "date"
require "j_law_ruby"

FIXTURES = JSON.parse(
  File.read(File.join(__dir__, "../../../tests/fixtures/income_tax_deductions.json"))
)

def build_input(input)
  {
    spouse: input["spouse"] && {
      spouse_total_income_amount: input["spouse"]["spouse_total_income_amount"],
      is_same_household: input["spouse"]["is_same_household"],
      is_elderly: input["spouse"]["is_elderly"],
    },
    dependent: {
      general_count: input.fetch("dependent", {}).fetch("general_count", 0),
      specific_count: input.fetch("dependent", {}).fetch("specific_count", 0),
      elderly_cohabiting_count: input.fetch("dependent", {}).fetch("elderly_cohabiting_count", 0),
      elderly_other_count: input.fetch("dependent", {}).fetch("elderly_other_count", 0),
    },
    social_insurance_premium_paid: input.fetch("social_insurance_premium_paid", 0),
    medical: input["medical"] && {
      medical_expense_paid: input["medical"]["medical_expense_paid"],
      reimbursed_amount: input["medical"]["reimbursed_amount"],
    },
    life_insurance: input["life_insurance"] && {
      new_general_paid_amount: input["life_insurance"]["new_general_paid_amount"],
      new_individual_pension_paid_amount: input["life_insurance"]["new_individual_pension_paid_amount"],
      new_care_medical_paid_amount: input["life_insurance"]["new_care_medical_paid_amount"],
      old_general_paid_amount: input["life_insurance"]["old_general_paid_amount"],
      old_individual_pension_paid_amount: input["life_insurance"]["old_individual_pension_paid_amount"],
    },
    donation: input["donation"] && {
      qualified_donation_amount: input["donation"]["qualified_donation_amount"],
    },
  }
end

class TestIncomeTaxDeductionsFixtures < Minitest::Test
  FIXTURES["income_tax_deductions"].each do |c|
    define_method("test_#{c['id']}") do
      inp = c["input"]
      exp = c["expected"]

      result = JLawRuby::IncomeTax.calc_income_deductions(
        inp["total_income_amount"],
        Date.parse(inp["date"]),
        **build_input(inp)
      )

      assert_equal exp["total_income_amount"], result.total_income_amount
      assert_equal exp["total_deductions"], result.total_deductions
      assert_equal exp["taxable_income_before_truncation"], result.taxable_income_before_truncation
      assert_equal exp["taxable_income"], result.taxable_income
    end
  end
end

class TestIncomeTaxAssessmentFixtures < Minitest::Test
  FIXTURES["income_tax_assessment"].each do |c|
    define_method("test_#{c['id']}") do
      inp = c["input"]
      exp = c["expected"]

      result = JLawRuby::IncomeTax.calc_income_tax_assessment(
        inp["total_income_amount"],
        Date.parse(inp["date"]),
        apply_reconstruction_tax: inp["apply_reconstruction_tax"],
        **build_input(inp)
      )

      assert_equal exp["taxable_income"], result.deductions.taxable_income
      assert_equal exp["base_tax"], result.tax.base_tax
      assert_equal exp["reconstruction_tax"], result.tax.reconstruction_tax
      assert_equal exp["total_tax"], result.tax.total_tax
    end
  end
end
