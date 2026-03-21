# frozen_string_literal: true

require "minitest/autorun"
require "j_law_ruby"

class TestCFFIAdapter < Minitest::Test
  def test_ffi_version_matches
    assert_equal 4, JLawRuby::Internal::CFFI.ffi_version
  end

  def test_compiled_library_is_loaded_from_gem_path
    expected_path = File.expand_path(
      "../lib/j_law_ruby/native/#{JLawRuby::BuildSupport.shared_library_filename}",
      __dir__
    )

    assert_equal expected_path, JLawRuby::Internal::CFFI.library_path
  end

  def test_fixed_length_strings_are_restored
    brokerage = JLawRuby::Internal::CFFI.calc_brokerage_fee(5_000_000, 2024, 8, 1, false, false)
    assert_equal %w[tier1 tier2 tier3], brokerage.breakdown.map(&:label)

    stamp = JLawRuby::Internal::CFFI.calc_stamp_tax(2, 5_000_000, 2024, 8, 1, 0)
    refute_empty stamp.rule_label

    withholding = JLawRuby::Internal::CFFI.calc_withholding_tax(1_500_000, 0, 2026, 1, 1, 2, false)
    assert_equal 2, withholding.breakdown.length
  end

  def test_error_path_raises_runtime_error
    error = assert_raises(RuntimeError) do
      JLawRuby::Internal::CFFI.calc_consumption_tax(100_000, 2016, 1, 1, true)
    end

    refute_empty error.message
  end

  def test_invalid_date_parts_are_rejected
    error = assert_raises(ArgumentError) do
      JLawRuby::Internal::CFFI.calc_brokerage_fee(5_000_000, 2024, 13, 1, false, false)
    end

    assert_match(/2024-13-01/, error.message)
  end
end
