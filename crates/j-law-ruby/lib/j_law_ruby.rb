# frozen_string_literal: true

require "date"
require_relative "j_law_ruby/build_support"
require_relative "j_law_ruby/c_ffi"

# 日本の法令に基づく各種計算を提供するモジュール。
#
# `j-law-c-ffi` の C FFI を ffi gem 経由でラップし、
# Ruby Date オブジェクトを受け取るインターフェースを提供する。
module JLawRuby
  # ── 消費税 ──────────────────────────────────────────────────────────────────

  module ConsumptionTax
    # 消費税の計算結果。
    class ConsumptionTaxResult
      attr_reader :tax_amount, :amount_with_tax, :amount_without_tax,
                  :applied_rate_numer, :applied_rate_denom

      def initialize(r)
        @tax_amount = r.tax_amount
        @amount_with_tax = r.amount_with_tax
        @amount_without_tax = r.amount_without_tax
        @applied_rate_numer = r.applied_rate_numer
        @applied_rate_denom = r.applied_rate_denom
        @is_reduced_rate = r.is_reduced_rate
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

      r = Internal::CFFI.calc_consumption_tax(amount, date.year, date.month, date.day, is_reduced_rate)
      ConsumptionTaxResult.new(r)
    end
  end

  # ── 不動産（媒介報酬） ───────────────────────────────────────────────────────

  module RealEstate
    # 媒介報酬の計算結果。
    class BrokerageFeeResult
      attr_reader :total_without_tax, :total_with_tax, :tax_amount, :breakdown

      def initialize(r)
        @total_without_tax = r.total_without_tax
        @total_with_tax = r.total_with_tax
        @tax_amount = r.tax_amount
        @low_cost_special_applied = r.low_cost_special_applied
        @breakdown = r.breakdown.map do |step|
          {
            label: step.label,
            base_amount: step.base_amount,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            result: step.result,
          }
        end
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

      r = Internal::CFFI.calc_brokerage_fee(
        price, date.year, date.month, date.day,
        is_low_cost_vacant_house, is_seller
      )
      BrokerageFeeResult.new(r)
    end
  end

  # ── 所得税 ──────────────────────────────────────────────────────────────────

  module IncomeTax
    # 所得税の計算結果。
    class IncomeTaxResult
      attr_reader :base_tax, :reconstruction_tax, :total_tax, :breakdown

      def initialize(r)
        @base_tax = r.base_tax
        @reconstruction_tax = r.reconstruction_tax
        @total_tax = r.total_tax
        @reconstruction_tax_applied = r.reconstruction_tax_applied
        @breakdown = r.breakdown.map do |step|
          {
            label: step.label,
            taxable_income: step.taxable_income,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            deduction: step.deduction,
            result: step.result,
          }
        end
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

    # 所得控除の計算結果。
    class IncomeDeductionResult
      attr_reader :total_income_amount, :total_deductions,
                  :taxable_income_before_truncation, :taxable_income, :breakdown

      def initialize(r)
        @total_income_amount = r.total_income_amount
        @total_deductions = r.total_deductions
        @taxable_income_before_truncation = r.taxable_income_before_truncation
        @taxable_income = r.taxable_income
        @breakdown = r.breakdown.map do |line|
          {
            kind: line.kind,
            label: line.label,
            amount: line.amount,
          }
        end
      end

      def inspect
        "#<JLawRuby::IncomeTax::IncomeDeductionResult " \
          "total_income_amount=#{@total_income_amount} " \
          "total_deductions=#{@total_deductions} " \
          "taxable_income=#{@taxable_income}>"
      end

      alias to_s inspect
    end

    # 所得控除から所得税額までの通し計算結果。
    class IncomeTaxAssessmentResult
      attr_reader :deductions, :tax

      def initialize(r)
        @deductions = IncomeDeductionResult.new(r.deductions)
        @tax = IncomeTaxResult.new(r.tax)
      end

      def inspect
        "#<JLawRuby::IncomeTax::IncomeTaxAssessmentResult " \
          "taxable_income=#{@deductions.taxable_income} " \
          "total_tax=#{@tax.total_tax}>"
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

      r = Internal::CFFI.calc_income_tax(
        taxable_income, date.year, date.month, date.day,
        apply_reconstruction_tax
      )
      IncomeTaxResult.new(r)
    end

    # 所得控除を計算し、課税所得金額までを返す。
    def self.calc_income_deductions(
      total_income_amount,
      date,
      spouse: nil,
      dependent: {},
      social_insurance_premium_paid: 0,
      medical: nil,
      life_insurance: nil,
      donation: nil
    )
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      r = Internal::CFFI.calc_income_deductions(
        total_income_amount: total_income_amount,
        year: date.year,
        month: date.month,
        day: date.day,
        spouse: spouse,
        dependent: dependent,
        social_insurance_premium_paid: social_insurance_premium_paid,
        medical: medical,
        life_insurance: life_insurance,
        donation: donation
      )
      IncomeDeductionResult.new(r)
    end

    # 所得控除から所得税額までを通しで計算する。
    def self.calc_income_tax_assessment(
      total_income_amount,
      date,
      apply_reconstruction_tax: true,
      spouse: nil,
      dependent: {},
      social_insurance_premium_paid: 0,
      medical: nil,
      life_insurance: nil,
      donation: nil
    )
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      r = Internal::CFFI.calc_income_tax_assessment(
        {
          total_income_amount: total_income_amount,
          year: date.year,
          month: date.month,
          day: date.day,
          spouse: spouse,
          dependent: dependent,
          social_insurance_premium_paid: social_insurance_premium_paid,
          medical: medical,
          life_insurance: life_insurance,
          donation: donation,
        },
        apply_reconstruction_tax
      )
      IncomeTaxAssessmentResult.new(r)
    end
  end

  # ── 印紙税 ──────────────────────────────────────────────────────────────────

  module StampTax
    DOCUMENT_KIND_MAP = {
      real_estate_transfer: Internal::CFFI::STAMP_TAX_DOCUMENT_KIND_REAL_ESTATE_TRANSFER,
      construction_contract: Internal::CFFI::STAMP_TAX_DOCUMENT_KIND_CONSTRUCTION_CONTRACT
    }.freeze

    # 印紙税の計算結果。
    class StampTaxResult
      attr_reader :tax_amount, :bracket_label

      def initialize(r)
        @tax_amount = r.tax_amount
        @bracket_label = r.bracket_label
        @reduced_rate_applied = r.reduced_rate_applied
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
    # @param document_kind [:real_estate_transfer, :construction_contract, String] 文書種別
    # @return [StampTaxResult]
    # @raise [TypeError] date または document_kind の型が不正な場合
    # @raise [RuntimeError] 計算エラーが発生した場合
    def self.calc_stamp_tax(contract_amount, date, is_reduced_rate_applicable = false,
                           document_kind: :real_estate_transfer)
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end
      document_kind_value = normalize_document_kind(document_kind)

      r = Internal::CFFI.calc_stamp_tax(
        contract_amount, date.year, date.month, date.day,
        is_reduced_rate_applicable, document_kind_value
      )
      StampTaxResult.new(r)
    end

    def self.normalize_document_kind(document_kind)
      key = case document_kind
            when Symbol
              document_kind
            when String
              document_kind.to_sym
            else
              raise TypeError,
                    "document_kind には Symbol または String を指定してください " \
                    "(got #{document_kind.class})"
            end

      DOCUMENT_KIND_MAP.fetch(key)
    rescue KeyError
      raise ArgumentError,
            "document_kind は :real_estate_transfer または :construction_contract を指定してください"
    end
    private_class_method :normalize_document_kind
  end

  # ── 源泉徴収 ────────────────────────────────────────────────────────────────

  module WithholdingTax
    MANUSCRIPT_AND_LECTURE = 1
    PROFESSIONAL_FEE = 2
    EXCLUSIVE_CONTRACT_FEE = 3

    # 源泉徴収税額の計算結果。
    class WithholdingTaxResult
      attr_reader :gross_payment_amount, :taxable_payment_amount, :tax_amount,
                  :net_payment_amount, :category, :breakdown

      def initialize(r)
        @gross_payment_amount = r.gross_payment_amount
        @taxable_payment_amount = r.taxable_payment_amount
        @tax_amount = r.tax_amount
        @net_payment_amount = r.net_payment_amount
        @category = self.class.category_to_symbol(r.category)
        @submission_prize_exempted = r.submission_prize_exempted
        @breakdown = r.breakdown.map do |step|
          {
            label: step.label,
            base_amount: step.base_amount,
            rate_numer: step.rate_numer,
            rate_denom: step.rate_denom,
            result: step.result,
          }
        end
      end

      def self.category_to_symbol(category)
        case category
        when MANUSCRIPT_AND_LECTURE then :manuscript_and_lecture
        when PROFESSIONAL_FEE then :professional_fee
        when EXCLUSIVE_CONTRACT_FEE then :exclusive_contract_fee
        else category
        end
      end

      def submission_prize_exempted?
        @submission_prize_exempted
      end

      def inspect
        "#<JLawRuby::WithholdingTax::WithholdingTaxResult " \
          "gross_payment_amount=#{@gross_payment_amount} " \
          "taxable_payment_amount=#{@taxable_payment_amount} " \
          "tax_amount=#{@tax_amount} " \
          "net_payment_amount=#{@net_payment_amount} " \
          "category=#{@category.inspect} " \
          "submission_prize_exempted=#{@submission_prize_exempted}>"
      end

      alias to_s inspect
    end

    def self.calc_withholding_tax(
      payment_amount,
      date,
      category,
      is_submission_prize: false,
      separated_consumption_tax_amount: 0
    )
      unless date.is_a?(::Date) || date.is_a?(::DateTime)
        raise TypeError,
              "date には Date または DateTime を指定してください (got #{date.class})"
      end

      r = Internal::CFFI.calc_withholding_tax(
        payment_amount,
        separated_consumption_tax_amount,
        date.year,
        date.month,
        date.day,
        normalize_category(category),
        is_submission_prize
      )
      WithholdingTaxResult.new(r)
    end

    def self.normalize_category(category)
      case category
      when Integer then category
      when String then normalize_category(category.to_sym)
      when :manuscript_and_lecture then MANUSCRIPT_AND_LECTURE
      when :professional_fee then PROFESSIONAL_FEE
      when :exclusive_contract_fee then EXCLUSIVE_CONTRACT_FEE
      else
        raise ArgumentError, "unknown withholding tax category: #{category.inspect}"
      end
    end
  end
end
