# frozen_string_literal: true

require "date"
require_relative "j_law_ruby/j_law_uniffi"

# 日本の法令に基づく各種計算を提供するモジュール。
#
# UniFFI が生成した Ruby バインディング（JLawUniffi）を薄くラップし、
# Ruby Date オブジェクトを受け取るインターフェースを提供する。
module JLawRuby
  # ── 消費税 ──────────────────────────────────────────────────────────────────

  module ConsumptionTax
    # 消費税法第29条に基づく消費税額を計算する。
    #
    # @param amount [Integer] 課税標準額（税抜き・円）
    # @param date [Date] 基準日
    # @param is_reduced_rate [true, false] 軽減税率フラグ
    # @return [JLawUniffi::ConsumptionTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_consumption_tax(amount, date, is_reduced_rate = false)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      JLawUniffi.calc_consumption_tax(amount, date.year, date.month, date.day, is_reduced_rate)
    rescue JLawUniffi::JLawError => e
      raise RuntimeError, e.message
    end
  end

  # ── 不動産（媒介報酬） ───────────────────────────────────────────────────────

  module RealEstate
    # 宅建業法第46条に基づく媒介報酬を計算する。
    #
    # @param price [Integer] 売買価格（円）
    # @param date [Date] 基準日
    # @param is_low_cost_vacant_house [true, false] 低廉な空き家特例フラグ
    # @param is_seller [true, false] 売主側フラグ
    # @return [JLawUniffi::BrokerageFeeResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_brokerage_fee(price, date, is_low_cost_vacant_house, is_seller)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      JLawUniffi.calc_brokerage_fee(
        price, date.year, date.month, date.day,
        is_low_cost_vacant_house, is_seller
      )
    rescue JLawUniffi::JLawError => e
      raise RuntimeError, e.message
    end
  end

  # ── 所得税 ──────────────────────────────────────────────────────────────────

  module IncomeTax
    # 所得税法第89条に基づく所得税額を計算する。
    #
    # @param taxable_income [Integer] 課税所得金額（円）
    # @param date [Date] 基準日
    # @param apply_reconstruction_tax [true, false] 復興特別所得税を適用するか
    # @return [JLawUniffi::IncomeTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_income_tax(taxable_income, date, apply_reconstruction_tax)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      JLawUniffi.calc_income_tax(
        taxable_income, date.year, date.month, date.day,
        apply_reconstruction_tax
      )
    rescue JLawUniffi::JLawError => e
      raise RuntimeError, e.message
    end
  end

  # ── 印紙税 ──────────────────────────────────────────────────────────────────

  module StampTax
    # 印紙税法 別表第一に基づく印紙税額を計算する。
    #
    # @param contract_amount [Integer] 契約金額（円）
    # @param date [Date] 契約書作成日
    # @param is_reduced_rate_applicable [true, false] 軽減税率適用フラグ
    # @return [JLawUniffi::StampTaxResult]
    # @raise [TypeError] date が Date / DateTime 以外の場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_stamp_tax(contract_amount, date, is_reduced_rate_applicable)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      JLawUniffi.calc_stamp_tax(
        contract_amount, date.year, date.month, date.day,
        is_reduced_rate_applicable
      )
    rescue JLawUniffi::JLawError => e
      raise RuntimeError, e.message
    end
  end
end
