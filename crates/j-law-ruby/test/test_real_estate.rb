# frozen_string_literal: true

# 宅建業法第46条に基づく媒介報酬計算のテスト。
#
# 法的根拠: 宅地建物取引業法 第46条第1項 / 国土交通省告示
# テストケースは tests/fixtures/real_estate.json から読み込む。
#
# 実行方法:
#   rake compile && ruby -Ilib test/test_real_estate.rb

require "minitest/autorun"
require "json"
require "j_law_ruby"

REAL_ESTATE_FIXTURES = JSON.parse(
  File.read(File.join(__dir__, "../../../tests/fixtures/real_estate.json"))
)

# ─── データ駆動テスト ─────────────────────────────────────────────────────────

class TestBrokerageFeeFixtures < Minitest::Test
  REAL_ESTATE_FIXTURES["brokerage_fee"].each do |c|
    define_method("test_#{c['id']}") do
      inp = c["input"]
      exp = c["expected"]

      r = JLawRuby::RealEstate.calc_brokerage_fee(
        inp["price"], inp["year"], inp["month"], inp["day"],
        inp["is_low_cost_vacant_house"]
      )

      if exp.key?("total_without_tax")
        assert_equal exp["total_without_tax"], r.total_without_tax, "#{c['id']}: total_without_tax"
      end
      if exp.key?("tax_amount")
        assert_equal exp["tax_amount"], r.tax_amount, "#{c['id']}: tax_amount"
      end
      if exp.key?("total_with_tax")
        assert_equal exp["total_with_tax"], r.total_with_tax, "#{c['id']}: total_with_tax"
      end
      if exp.key?("low_cost_special_applied")
        if exp["low_cost_special_applied"]
          assert r.low_cost_special_applied?, "#{c['id']}: low_cost_special_applied"
        else
          refute r.low_cost_special_applied?, "#{c['id']}: low_cost_special_applied"
        end
      end
      if exp.key?("breakdown_results")
        actual = r.breakdown.map { |step| step[:result] }
        assert_equal exp["breakdown_results"], actual, "#{c['id']}: breakdown_results"
      end
    end
  end
end

# ─── 言語固有テスト ──────────────────────────────────────────────────────────

class TestBrokerageFeeLanguageSpecific < Minitest::Test
  def test_error_date_out_of_range
    err = assert_raises(RuntimeError) do
      JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2019, 9, 30, false)
    end
    assert_match(/2019-09-30/, err.message)
  end

  def test_breakdown_fields
    r = JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2024, 8, 1, false)
    r.breakdown.each do |step|
      refute_empty step[:label]
      assert_operator step[:rate_denom], :>, 0
    end
  end

  def test_inspect
    r = JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2024, 8, 1, false)
    assert_match(/BrokerageFeeResult/, r.inspect)
  end
end
