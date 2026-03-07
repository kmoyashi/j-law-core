# frozen_string_literal: true

require "date"
require_relative "j_law_ruby/j_law_ruby"

# 日本の法令に基づく各種計算を提供するモジュール。
#
# ネイティブ拡張（JLawRubyNative）を薄くラップし、
# Ruby Date オブジェクトを受け取るインターフェースを提供する。
module JLawRuby
  # ── 消費税 ──────────────────────────────────────────────────────────────────

  module ConsumptionTax
    # 消費税の計算結果。
    class ConsumptionTaxResult
      attr_reader :tax_amount, :amount_with_tax, :amount_without_tax,
                  :applied_rate_numer, :applied_rate_denom

      def initialize(h)
        @tax_amount = h[:tax_amount]
        @amount_with_tax = h[:amount_with_tax]
        @amount_without_tax = h[:amount_without_tax]
        @applied_rate_numer = h[:applied_rate_numer]
        @applied_rate_denom = h[:applied_rate_denom]
        @is_reduced_rate = h[:is_reduced_rate] == 1
      end

      # 軽減税率が適用されたか。
      def is_reduced_rate?
        @is_reduced_rate
      end

      def inspect
        "#<JLawRuby::ConsumptionTax::ConsumptionTaxResult " \
          "tax_amount=#{@tax_amount} " \
          "amount_with_tax=#{@amount_with_tax} " \
          "amount_without_tax=#{@amount_without_tax} " \
          "applied_rate=#{@applied_rate_numer}/#{@applied_rate_denom} " \
          "is_reduced_rate=#{@is_reduced_rate}>"
      end

      alias to_s inspect
    end

    # 消費税法第29条に基づく消費税額を計算する。
    #
    # @param amount [Integer] 課税標準額（税抜き・円）
    # @param date [Date] 基準日
    # @param is_reduced_rate [true, false] 軽減税率フラグ
    # @return [ConsumptionTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_consumption_tax(amount, date, is_reduced_rate = false)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      h = JLawRubyNative.calc_consumption_tax(
        amount, date.year, date.month, date.day,
        is_reduced_rate ? 1 : 0
      )
      ConsumptionTaxResult.new(h)
    end
  end

  # ── 不動産（媒介報酬） ───────────────────────────────────────────────────────

  module RealEstate
    # 媒介報酬の計算結果。
    class BrokerageFeeResult
      attr_reader :total_without_tax, :total_with_tax, :tax_amount, :breakdown

      def initialize(h)
        @total_without_tax = h[:total_without_tax]
        @total_with_tax = h[:total_with_tax]
        @tax_amount = h[:tax_amount]
        @low_cost_special_applied = h[:low_cost_special_applied] == 1
        @breakdown = h[:breakdown]
      end

      # 低廉な空き家特例が適用されたか。
      def low_cost_special_applied?
        @low_cost_special_applied
      end

      def inspect
        "#<JLawRuby::RealEstate::BrokerageFeeResult " \
          "total_without_tax=#{@total_without_tax} " \
          "total_with_tax=#{@total_with_tax} " \
          "tax_amount=#{@tax_amount} " \
          "low_cost_special_applied=#{@low_cost_special_applied}>"
      end

      alias to_s inspect
    end

    # 宅建業法第46条に基づく媒介報酬を計算する。
    #
    # @param price [Integer] 売買価格（円）
    # @param date [Date] 基準日
    # @param is_low_cost_vacant_house [true, false] 低廉な空き家特例フラグ
    # @param is_seller [true, false] 売主側フラグ
    # @return [BrokerageFeeResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_brokerage_fee(price, date, is_low_cost_vacant_house, is_seller)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      h = JLawRubyNative.calc_brokerage_fee(
        price, date.year, date.month, date.day,
        is_low_cost_vacant_house ? 1 : 0,
        is_seller ? 1 : 0
      )
      BrokerageFeeResult.new(h)
    end
  end

  # ── 所得税 ──────────────────────────────────────────────────────────────────

  module IncomeTax
    # 所得税の計算結果。
    class IncomeTaxResult
      attr_reader :base_tax, :reconstruction_tax, :total_tax, :breakdown

      def initialize(h)
        @base_tax = h[:base_tax]
        @reconstruction_tax = h[:reconstruction_tax]
        @total_tax = h[:total_tax]
        @reconstruction_tax_applied = h[:reconstruction_tax_applied] == 1
        @breakdown = h[:breakdown]
      end

      # 復興特別所得税が適用されたか。
      def reconstruction_tax_applied?
        @reconstruction_tax_applied
      end

      def inspect
        "#<JLawRuby::IncomeTax::IncomeTaxResult " \
          "base_tax=#{@base_tax} " \
          "reconstruction_tax=#{@reconstruction_tax} " \
          "total_tax=#{@total_tax} " \
          "reconstruction_tax_applied=#{@reconstruction_tax_applied}>"
      end

      alias to_s inspect
    end

    # 所得税法第89条に基づく所得税額を計算する。
    #
    # @param taxable_income [Integer] 課税所得金額（円）
    # @param date [Date] 基準日
    # @param apply_reconstruction_tax [true, false] 復興特別所得税を適用するか
    # @return [IncomeTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_income_tax(taxable_income, date, apply_reconstruction_tax)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      h = JLawRubyNative.calc_income_tax(
        taxable_income, date.year, date.month, date.day,
        apply_reconstruction_tax ? 1 : 0
      )
      IncomeTaxResult.new(h)
    end
  end

  # ── 印紙税 ──────────────────────────────────────────────────────────────────

  module StampTax
    # 印紙税の計算結果。
    class StampTaxResult
      attr_reader :tax_amount, :bracket_label

      def initialize(h)
        @tax_amount = h[:tax_amount]
        @bracket_label = h[:bracket_label]
        @reduced_rate_applied = h[:reduced_rate_applied] == 1
      end

      # 軽減税率が適用されたか。
      def reduced_rate_applied?
        @reduced_rate_applied
      end

      def inspect
        "#<JLawRuby::StampTax::StampTaxResult " \
          "tax_amount=#{@tax_amount} " \
          "bracket_label=#{@bracket_label.inspect} " \
          "reduced_rate_applied=#{@reduced_rate_applied}>"
      end

      alias to_s inspect
    end

    # 印紙税法 別表第一に基づく印紙税額を計算する。
    #
    # @param contract_amount [Integer] 契約金額（円）
    # @param date [Date] 契約書作成日
    # @param is_reduced_rate_applicable [true, false] 軽減税率適用フラグ
    # @return [StampTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_stamp_tax(contract_amount, date, is_reduced_rate_applicable)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      h = JLawRubyNative.calc_stamp_tax(
        contract_amount, date.year, date.month, date.day,
        is_reduced_rate_applicable ? 1 : 0
      )
      StampTaxResult.new(h)
    end
  end
end
