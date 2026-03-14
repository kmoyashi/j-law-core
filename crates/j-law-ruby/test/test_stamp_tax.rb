# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "date"
require "j_law_ruby"

class TestStampTax < Minitest::Test
  FIXTURES = JSON.parse(File.read(File.join(__dir__, "../../../tests/fixtures/stamp_tax.json")))

  FIXTURES["stamp_tax"].each do |tc|
    define_method("test_#{tc['id']}") do
      inp = tc["input"]
      exp = tc["expected"]

      date = Date.parse(inp["date"])
      result = JLawRuby::StampTax.calc_stamp_tax(
        inp["document_code"],
        inp["stated_amount"],
        date,
        flags: inp["flags"]
      )

      assert_equal exp["tax_amount"], result.tax_amount, "#{tc['id']}: tax_amount"
      assert_equal exp["rule_label"], result.rule_label, "#{tc['id']}: rule_label"
      if exp["applied_special_rule"].nil?
        assert_nil result.applied_special_rule, "#{tc['id']}: applied_special_rule"
      else
        assert_equal exp["applied_special_rule"], result.applied_special_rule, "#{tc['id']}: applied_special_rule"
      end
    end
  end

  def test_error_date_out_of_range
    err = assert_raises(RuntimeError) do
      JLawRuby::StampTax.calc_stamp_tax("article1_real_estate_transfer", 5_000_000, Date.new(2014, 3, 31))
    end
    assert_match(/2014-03-31/, err.message)
  end

  def test_inspect
    result = JLawRuby::StampTax.calc_stamp_tax("article1_real_estate_transfer", 5_000_000, Date.new(2024, 8, 1))
    assert_match(/StampTaxResult/, result.inspect)
  end

  def test_type_error_invalid_date
    assert_raises(TypeError) do
      JLawRuby::StampTax.calc_stamp_tax("article1_real_estate_transfer", 5_000_000, "2024-08-01")
    end
  end

  def test_invalid_document_code
    assert_raises(ArgumentError) do
      JLawRuby::StampTax.calc_stamp_tax("invalid_code", 5_000_000, Date.new(2024, 8, 1))
    end
  end

  def test_invalid_flag
    assert_raises(ArgumentError) do
      JLawRuby::StampTax.calc_stamp_tax(
        "article17_sales_receipt",
        70_000,
        Date.new(2024, 8, 1),
        flags: ["invalid_flag"]
      )
    end
  end
end
