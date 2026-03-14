# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "date"
require "j_law_ruby"

class TestWithholdingTax < Minitest::Test
  FIXTURES = JSON.parse(File.read(File.join(__dir__, "../../../tests/fixtures/withholding_tax.json")))

  FIXTURES["withholding_tax"].each do |tc|
    define_method("test_#{tc['id']}") do
      inp = tc["input"]
      exp = tc["expected"]

      result = JLawRuby::WithholdingTax.calc_withholding_tax(
        inp["payment_amount"],
        Date.parse(inp["date"]),
        inp["category"],
        is_submission_prize: inp["is_submission_prize"],
        separated_consumption_tax_amount: inp["separated_consumption_tax_amount"]
      )

      assert_equal exp["taxable_payment_amount"], result.taxable_payment_amount
      assert_equal exp["tax_amount"], result.tax_amount
      assert_equal exp["net_payment_amount"], result.net_payment_amount
      assert_equal exp["submission_prize_exempted"], result.submission_prize_exempted?
    end
  end

  def test_out_of_range_date
    assert_raises(RuntimeError) do
      JLawRuby::WithholdingTax.calc_withholding_tax(
        100_000,
        Date.new(2012, 12, 31),
        :manuscript_and_lecture
      )
    end
  end

  def test_breakdown_fields
    result = JLawRuby::WithholdingTax.calc_withholding_tax(
      1_500_000,
      Date.new(2026, 1, 1),
      :professional_fee
    )
    assert_equal 2, result.breakdown.length
    refute_empty result.breakdown.first[:label]
  end

  def test_inspect
    result = JLawRuby::WithholdingTax.calc_withholding_tax(
      100_000,
      Date.new(2026, 1, 1),
      :manuscript_and_lecture
    )
    assert_match(/WithholdingTaxResult/, result.inspect)
  end

  def test_type_error_invalid_date
    assert_raises(TypeError) do
      JLawRuby::WithholdingTax.calc_withholding_tax(100_000, "2026-01-01", :professional_fee)
    end
  end
end
