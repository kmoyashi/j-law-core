# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "j_law_ruby"

# 印紙税法 別表第一に基づく印紙税額計算のテスト。
#
# 法的根拠: 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条
# テストケースは tests/fixtures/stamp_tax.json から読み込む。
class TestStampTax < Minitest::Test
  FIXTURES = JSON.parse(File.read(File.join(__dir__, "../../../tests/fixtures/stamp_tax.json")))

  # ─── データ駆動テスト ─────────────────────────────────────────────────────

  FIXTURES["stamp_tax"].each do |tc|
    define_method("test_#{tc['id']}") do
      inp = tc["input"]
      exp = tc["expected"]

      result = JLawRuby::StampTax.calc_stamp_tax(
        inp["contract_amount"],
        inp["year"],
        inp["month"],
        inp["day"],
        inp["is_reduced_rate_applicable"]
      )

      assert_equal exp["tax_amount"], result.tax_amount, "#{tc['id']}: tax_amount"
      assert_equal exp["reduced_rate_applied"], result.reduced_rate_applied?, "#{tc['id']}: reduced_rate_applied"
    end
  end

  # ─── 言語固有テスト ───────────────────────────────────────────────────────

  def test_error_date_out_of_range
    err = assert_raises(RuntimeError) do
      JLawRuby::StampTax.calc_stamp_tax(5_000_000, 2014, 3, 31, false)
    end
    assert_match(/2014-03-31/, err.message)
  end

  def test_bracket_label_present
    result = JLawRuby::StampTax.calc_stamp_tax(5_000_000, 2024, 8, 1, false)
    refute_empty result.bracket_label
  end

  def test_inspect
    result = JLawRuby::StampTax.calc_stamp_tax(5_000_000, 2024, 8, 1, false)
    assert_match(/StampTaxResult/, result.inspect)
  end
end
