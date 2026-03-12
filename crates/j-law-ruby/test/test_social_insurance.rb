# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "date"
require "j_law_ruby"

class TestSocialInsurance < Minitest::Test
  FIXTURES = JSON.parse(File.read(File.join(__dir__, "../../../tests/fixtures/social_insurance.json")))

  FIXTURES["social_insurance"].each do |tc|
    define_method("test_#{tc['id']}") do
      inp = tc["input"]
      exp = tc["expected"]

      result = JLawRuby::SocialInsurance.calc_social_insurance(
        inp["standard_monthly_remuneration"],
        Date.parse(inp["date"]),
        inp["prefecture_code"],
        inp["is_care_insurance_applicable"]
      )

      assert_equal exp["health_related_amount"], result.health_related_amount
      assert_equal exp["pension_amount"], result.pension_amount
      assert_equal exp["total_amount"], result.total_amount
      assert_equal exp["care_insurance_applied"], result.care_insurance_applied?
    end
  end

  def test_invalid_standard_monthly_remuneration
    err = assert_raises(RuntimeError) do
      JLawRuby::SocialInsurance.calc_social_insurance(305_000, Date.new(2026, 3, 1), 13, false)
    end
    assert_match(/標準報酬月額/, err.message)
  end
end
