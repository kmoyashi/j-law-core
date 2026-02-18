# frozen_string_literal: true

# 所得税法第89条に基づく所得税計算のテスト。
#
# 法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条
# テストケースは tests/fixtures/income_tax.json から読み込む。
#
# 実行方法:
#   rake compile && ruby -Ilib test/test_income_tax.rb

require "minitest/autorun"
require "json"
require "j_law_core"

INCOME_TAX_FIXTURES = JSON.parse(
  File.read(File.join(__dir__, "../../../tests/fixtures/income_tax.json"))
)

# ─── データ駆動テスト ─────────────────────────────────────────────────────────

class TestIncomeTaxFixtures < Minitest::Test
  INCOME_TAX_FIXTURES["income_tax"].each do |c|
    define_method("test_#{c['id']}") do
      inp = c["input"]
      exp = c["expected"]

      r = JLawCore::IncomeTax.calc_income_tax(
        inp["taxable_income"], inp["year"], inp["month"], inp["day"],
        inp["apply_reconstruction_tax"]
      )

      assert_equal exp["base_tax"], r.base_tax, "#{c['id']}: base_tax"
      assert_equal exp["reconstruction_tax"], r.reconstruction_tax, "#{c['id']}: reconstruction_tax"
      assert_equal exp["total_tax"], r.total_tax, "#{c['id']}: total_tax"
      if exp["reconstruction_tax_applied"]
        assert r.reconstruction_tax_applied?, "#{c['id']}: reconstruction_tax_applied"
      else
        refute r.reconstruction_tax_applied?, "#{c['id']}: reconstruction_tax_applied"
      end
    end
  end
end

# ─── 言語固有テスト ──────────────────────────────────────────────────────────

class TestIncomeTaxLanguageSpecific < Minitest::Test
  def test_error_date_out_of_range
    assert_raises(RuntimeError) do
      JLawCore::IncomeTax.calc_income_tax(5_000_000, 2014, 12, 31, true)
    end
  end

  def test_breakdown_fields
    r = JLawCore::IncomeTax.calc_income_tax(5_000_000, 2024, 1, 1, true)
    refute_empty r.breakdown
    r.breakdown.each do |step|
      refute_empty step[:label]
      assert_operator step[:rate_denom], :>, 0
    end
  end

  def test_inspect
    r = JLawCore::IncomeTax.calc_income_tax(5_000_000, 2024, 1, 1, true)
    assert_match(/IncomeTaxResult/, r.inspect)
  end
end
