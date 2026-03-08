# frozen_string_literal: true

require "minitest/autorun"
require "j_law_ruby"

class TestCgoAdapter < Minitest::Test
  def test_abi_version_matches
    assert_equal 1, JLawRuby::Internal::Cgo.abi_version
  end

  def test_compiled_library_is_loaded_from_gem_path
    expected_path = File.expand_path(
      "../lib/j_law_ruby/native/#{JLawRuby::BuildSupport.shared_library_filename}",
      __dir__
    )

    assert_equal expected_path, JLawRuby::Internal::Cgo.library_path
  end

  def test_fixed_length_strings_are_restored
    brokerage = JLawRuby::Internal::Cgo.calc_brokerage_fee(5_000_000, 2024, 8, 1, false, false)
    assert_equal %w[tier1 tier2 tier3], brokerage.breakdown.map(&:label)

    stamp = JLawRuby::Internal::Cgo.calc_stamp_tax(5_000_000, 2024, 8, 1, false)
    refute_empty stamp.bracket_label
  end

  def test_error_path_raises_runtime_error
    error = assert_raises(RuntimeError) do
      JLawRuby::Internal::Cgo.calc_consumption_tax(100_000, 2016, 1, 1, true)
    end

    refute_empty error.message
  end
end
