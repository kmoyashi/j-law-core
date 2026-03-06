# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "j_law_ruby"

# 消費税法第29条に基づく消費税額計算のテスト。
#
# 法的根拠: 消費税法 第29条（税率）
# テストケースは tests/fixtures/consumption_tax.json から読み込む。
class TestConsumptionTax < Minitest::Test
  FIXTURES = JSON.parse(File.read(File.join(__dir__, "../../../tests/fixtures/consumption_tax.json")))

  # ─── データ駆動テスト ─────────────────────────────────────────────────────

  FIXTURES["consumption_tax"].each do |tc|
    define_method("test_#{tc['id']}") do
      inp = tc["input"]
      exp = tc["expected"]

      year, month, day = inp["date"].split("-").map(&:to_i)
      result = JLawRuby::ConsumptionTax.calc_consumption_tax(
        inp["amount"],
        year, month, day,
        inp["is_reduced_rate"]
      )

      assert_equal exp["tax_amount"],        result.tax_amount,        "#{tc['id']}: tax_amount"
      assert_equal exp["amount_with_tax"],   result.amount_with_tax,   "#{tc['id']}: amount_with_tax"
      assert_equal exp["amount_without_tax"], result.amount_without_tax, "#{tc['id']}: amount_without_tax"
      assert_equal exp["applied_rate_numer"], result.applied_rate_numer, "#{tc['id']}: applied_rate_numer"
      assert_equal exp["applied_rate_denom"], result.applied_rate_denom, "#{tc['id']}: applied_rate_denom"
      assert_equal exp["is_reduced_rate"],   result.is_reduced_rate?,  "#{tc['id']}: is_reduced_rate"
    end
  end

  # ─── 言語固有テスト ───────────────────────────────────────────────────────

  def test_error_reduced_rate_without_support
    err = assert_raises(RuntimeError) do
      JLawRuby::ConsumptionTax.calc_consumption_tax(100_000, 2016, 1, 1, true)
    end
    refute_nil err.message
  end

  def test_before_introduction_no_tax
    result = JLawRuby::ConsumptionTax.calc_consumption_tax(100_000, 1988, 1, 1, false)
    assert_equal 0, result.tax_amount
    assert_equal 100_000, result.amount_with_tax
  end

  def test_inspect
    result = JLawRuby::ConsumptionTax.calc_consumption_tax(100_000, 2024, 1, 1, false)
    assert_match(/ConsumptionTaxResult/, result.inspect)
  end
end
