# frozen_string_literal: true

require "date"
require "ffi"
require_relative "build_support"

module JLawRuby
  module Internal
    module CFFI
      extend FFI::Library

      FFI_VERSION = 4
      MAX_TIERS = 8
      MAX_DEDUCTION_LINES = 8
      LABEL_LEN = 64
      ERROR_BUF_LEN = 256
      UINT64_MAX = (1 << 64) - 1
      UINT32_MAX = (1 << 32) - 1
      UINT16_MAX = (1 << 16) - 1
      UINT8_MAX = (1 << 8) - 1
      GEM_ROOT = File.expand_path("../..", __dir__)
      LIBRARY_PATH = BuildSupport.resolve_shared_library_path(GEM_ROOT)

      BreakdownStepRecord = Struct.new(
        :label, :base_amount, :rate_numer, :rate_denom, :result,
        keyword_init: true
      )
      BrokerageFeeRecord = Struct.new(
        :total_without_tax, :total_with_tax, :tax_amount, :low_cost_special_applied, :breakdown,
        keyword_init: true
      )
      IncomeTaxStepRecord = Struct.new(
        :label, :taxable_income, :rate_numer, :rate_denom, :deduction, :result,
        keyword_init: true
      )
      IncomeTaxRecord = Struct.new(
        :base_tax, :reconstruction_tax, :total_tax, :reconstruction_tax_applied, :breakdown,
        keyword_init: true
      )
      IncomeDeductionLineRecord = Struct.new(
        :kind, :label, :amount,
        keyword_init: true
      )
      IncomeDeductionRecord = Struct.new(
        :total_income_amount, :total_deductions,
        :taxable_income_before_truncation, :taxable_income, :breakdown,
        keyword_init: true
      )
      IncomeTaxAssessmentRecord = Struct.new(
        :deductions, :tax,
        keyword_init: true
      )
      ConsumptionTaxRecord = Struct.new(
        :tax_amount, :amount_with_tax, :amount_without_tax,
        :applied_rate_numer, :applied_rate_denom, :is_reduced_rate,
        keyword_init: true
      )
      StampTaxRecord = Struct.new(
        :tax_amount, :rule_label, :applied_special_rule,
        keyword_init: true
      )
      WithholdingTaxRecord = Struct.new(
        :gross_payment_amount, :taxable_payment_amount, :tax_amount, :net_payment_amount,
        :category, :submission_prize_exempted, :breakdown,
        keyword_init: true
      )

      unless LIBRARY_PATH
        raise LoadError,
              "j-law-c-ffi shared library was not found. Run `bundle exec rake compile` first."
      end

      ffi_lib LIBRARY_PATH

      class BreakdownStepStruct < FFI::Struct
        layout :label, [:char, LABEL_LEN],
               :base_amount, :uint64,
               :rate_numer, :uint64,
               :rate_denom, :uint64,
               :result, :uint64
      end

      class BrokerageFeeStruct < FFI::Struct
        layout :total_without_tax, :uint64,
               :total_with_tax, :uint64,
               :tax_amount, :uint64,
               :low_cost_special_applied, :int,
               :breakdown_padding, [:char, 4],
               :breakdown_storage, [:char, BreakdownStepStruct.size * MAX_TIERS],
               :breakdown_len, :int
      end

      class IncomeTaxStepStruct < FFI::Struct
        layout :label, [:char, LABEL_LEN],
               :taxable_income, :uint64,
               :rate_numer, :uint64,
               :rate_denom, :uint64,
               :deduction, :uint64,
               :result, :uint64
      end

      class IncomeTaxStruct < FFI::Struct
        layout :base_tax, :uint64,
               :reconstruction_tax, :uint64,
               :total_tax, :uint64,
               :reconstruction_tax_applied, :int,
               :breakdown_padding, [:char, 4],
               :breakdown_storage, [:char, IncomeTaxStepStruct.size * MAX_TIERS],
               :breakdown_len, :int
      end

      class IncomeDeductionLineStruct < FFI::Struct
        layout :kind, :uint32,
               :label, [:char, LABEL_LEN],
               :amount, :uint64
      end

      class IncomeDeductionInputStruct < FFI::Struct
        layout :total_income_amount, :uint64,
               :year, :uint16,
               :month, :uint8,
               :day, :uint8,
               :has_spouse, :int,
               :spouse_total_income_amount, :uint64,
               :spouse_is_same_household, :int,
               :spouse_is_elderly, :int,
               :dependent_general_count, :uint64,
               :dependent_specific_count, :uint64,
               :dependent_elderly_cohabiting_count, :uint64,
               :dependent_elderly_other_count, :uint64,
               :social_insurance_premium_paid, :uint64,
               :has_medical, :int,
               :medical_expense_paid, :uint64,
               :medical_reimbursed_amount, :uint64,
               :has_life_insurance, :int,
               :life_new_general_paid_amount, :uint64,
               :life_new_individual_pension_paid_amount, :uint64,
               :life_new_care_medical_paid_amount, :uint64,
               :life_old_general_paid_amount, :uint64,
               :life_old_individual_pension_paid_amount, :uint64,
               :has_donation, :int,
               :donation_qualified_amount, :uint64
      end

      class IncomeDeductionStruct < FFI::Struct
        layout :total_income_amount, :uint64,
               :total_deductions, :uint64,
               :taxable_income_before_truncation, :uint64,
               :taxable_income, :uint64,
               :breakdown_storage, [:char, IncomeDeductionLineStruct.size * MAX_DEDUCTION_LINES],
               :breakdown_len, :int
      end

      class IncomeTaxAssessmentStruct < FFI::Struct
        layout :total_income_amount, :uint64,
               :total_deductions, :uint64,
               :taxable_income_before_truncation, :uint64,
               :taxable_income, :uint64,
               :base_tax, :uint64,
               :reconstruction_tax, :uint64,
               :total_tax, :uint64,
               :reconstruction_tax_applied, :int,
               :deduction_padding, [:char, 4],
               :deduction_breakdown_storage, [:char, IncomeDeductionLineStruct.size * MAX_DEDUCTION_LINES],
               :deduction_breakdown_len, :int,
               :tax_padding, [:char, 4],
               :tax_breakdown_storage, [:char, IncomeTaxStepStruct.size * MAX_TIERS],
               :tax_breakdown_len, :int
      end

      class ConsumptionTaxStruct < FFI::Struct
        layout :tax_amount, :uint64,
               :amount_with_tax, :uint64,
               :amount_without_tax, :uint64,
               :applied_rate_numer, :uint64,
               :applied_rate_denom, :uint64,
               :is_reduced_rate, :int
      end

      class StampTaxStruct < FFI::Struct
        layout :tax_amount, :uint64,
               :rule_label, [:char, LABEL_LEN],
               :applied_special_rule, [:char, LABEL_LEN]
      end

      class WithholdingTaxStruct < FFI::Struct
        layout :gross_payment_amount, :uint64,
               :taxable_payment_amount, :uint64,
               :tax_amount, :uint64,
               :net_payment_amount, :uint64,
               :category, :uint32,
               :submission_prize_exempted, :int,
               :breakdown_storage, [:char, BreakdownStepStruct.size * MAX_TIERS],
               :breakdown_len, :int
      end

      attach_function :j_law_c_ffi_version, [], :uint32
      attach_function :j_law_calc_brokerage_fee,
                      [:uint64, :uint16, :uint8, :uint8, :int, :int,
                       BrokerageFeeStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_income_tax,
                      [:uint64, :uint16, :uint8, :uint8, :int,
                       IncomeTaxStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_income_deductions,
                      [IncomeDeductionInputStruct.by_ref,
                       IncomeDeductionStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_income_tax_assessment,
                      [IncomeDeductionInputStruct.by_ref, :int,
                       IncomeTaxAssessmentStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_consumption_tax,
                      [:uint64, :uint16, :uint8, :uint8, :int,
                       ConsumptionTaxStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_stamp_tax,
                      [:uint32, :uint64, :int, :uint16, :uint8, :uint8, :uint64,
                       StampTaxStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_withholding_tax,
                      [:uint64, :uint64, :uint16, :uint8, :uint8, :uint32, :int,
                       WithholdingTaxStruct.by_ref, :pointer, :int],
                      :int

      actual_ffi_version = j_law_c_ffi_version
      if actual_ffi_version != FFI_VERSION
        raise LoadError,
              "j-law-c-ffi FFI version mismatch: expected #{FFI_VERSION}, got #{actual_ffi_version}"
      end

      module_function

      def library_path
        LIBRARY_PATH
      end

      def ffi_version
        j_law_c_ffi_version
      end

      def calc_brokerage_fee(price, year, month, day, is_low_cost_vacant_house, is_seller)
        validated_price = validate_u64(price, "price")
        validated_year, validated_month, validated_day = validate_date_parts(year, month, day)
        result = BrokerageFeeStruct.new

        call_with_error do |error_buf|
          j_law_calc_brokerage_fee(
            validated_price,
            validated_year,
            validated_month,
            validated_day,
            bool_to_c_int(is_low_cost_vacant_house),
            bool_to_c_int(is_seller),
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        BrokerageFeeRecord.new(
          total_without_tax: result[:total_without_tax],
          total_with_tax: result[:total_with_tax],
          tax_amount: result[:tax_amount],
          low_cost_special_applied: c_int_to_bool(result[:low_cost_special_applied]),
          breakdown: read_struct_array(
            result.pointer + BrokerageFeeStruct.offset_of(:breakdown_storage),
            BreakdownStepStruct,
            result[:breakdown_len]
          ).map do |step|
            BreakdownStepRecord.new(
              label: read_fixed_string(step, :label, LABEL_LEN),
              base_amount: step[:base_amount],
              rate_numer: step[:rate_numer],
              rate_denom: step[:rate_denom],
              result: step[:result]
            )
          end
        )
      end

      def calc_income_tax(taxable_income, year, month, day, apply_reconstruction_tax)
        validated_taxable_income = validate_u64(taxable_income, "taxable_income")
        validated_year, validated_month, validated_day = validate_date_parts(year, month, day)
        result = IncomeTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_income_tax(
            validated_taxable_income,
            validated_year,
            validated_month,
            validated_day,
            bool_to_c_int(apply_reconstruction_tax),
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        IncomeTaxRecord.new(
          base_tax: result[:base_tax],
          reconstruction_tax: result[:reconstruction_tax],
          total_tax: result[:total_tax],
          reconstruction_tax_applied: c_int_to_bool(result[:reconstruction_tax_applied]),
          breakdown: read_struct_array(
            result.pointer + IncomeTaxStruct.offset_of(:breakdown_storage),
            IncomeTaxStepStruct,
            result[:breakdown_len]
          ).map do |step|
            IncomeTaxStepRecord.new(
              label: read_fixed_string(step, :label, LABEL_LEN),
              taxable_income: step[:taxable_income],
              rate_numer: step[:rate_numer],
              rate_denom: step[:rate_denom],
              deduction: step[:deduction],
              result: step[:result]
            )
          end
        )
      end

      def calc_income_deductions(input)
        input_struct = build_income_deduction_input_struct(input)
        result = IncomeDeductionStruct.new

        call_with_error do |error_buf|
          j_law_calc_income_deductions(
            input_struct,
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        IncomeDeductionRecord.new(
          total_income_amount: result[:total_income_amount],
          total_deductions: result[:total_deductions],
          taxable_income_before_truncation: result[:taxable_income_before_truncation],
          taxable_income: result[:taxable_income],
          breakdown: read_struct_array(
            result.pointer + IncomeDeductionStruct.offset_of(:breakdown_storage),
            IncomeDeductionLineStruct,
            result[:breakdown_len],
            max_length: MAX_DEDUCTION_LINES
          ).map do |line|
            IncomeDeductionLineRecord.new(
              kind: line[:kind],
              label: read_fixed_string(line, :label, LABEL_LEN),
              amount: line[:amount]
            )
          end
        )
      end

      def calc_income_tax_assessment(input, apply_reconstruction_tax)
        input_struct = build_income_deduction_input_struct(input)
        result = IncomeTaxAssessmentStruct.new

        call_with_error do |error_buf|
          j_law_calc_income_tax_assessment(
            input_struct,
            bool_to_c_int(apply_reconstruction_tax),
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        deductions = IncomeDeductionRecord.new(
          total_income_amount: result[:total_income_amount],
          total_deductions: result[:total_deductions],
          taxable_income_before_truncation: result[:taxable_income_before_truncation],
          taxable_income: result[:taxable_income],
          breakdown: read_struct_array(
            result.pointer + IncomeTaxAssessmentStruct.offset_of(:deduction_breakdown_storage),
            IncomeDeductionLineStruct,
            result[:deduction_breakdown_len],
            max_length: MAX_DEDUCTION_LINES
          ).map do |line|
            IncomeDeductionLineRecord.new(
              kind: line[:kind],
              label: read_fixed_string(line, :label, LABEL_LEN),
              amount: line[:amount]
            )
          end
        )
        tax = IncomeTaxRecord.new(
          base_tax: result[:base_tax],
          reconstruction_tax: result[:reconstruction_tax],
          total_tax: result[:total_tax],
          reconstruction_tax_applied: c_int_to_bool(result[:reconstruction_tax_applied]),
          breakdown: read_struct_array(
            result.pointer + IncomeTaxAssessmentStruct.offset_of(:tax_breakdown_storage),
            IncomeTaxStepStruct,
            result[:tax_breakdown_len]
          ).map do |step|
            IncomeTaxStepRecord.new(
              label: read_fixed_string(step, :label, LABEL_LEN),
              taxable_income: step[:taxable_income],
              rate_numer: step[:rate_numer],
              rate_denom: step[:rate_denom],
              deduction: step[:deduction],
              result: step[:result]
            )
          end
        )

        IncomeTaxAssessmentRecord.new(
          deductions: deductions,
          tax: tax
        )
      end

      def calc_consumption_tax(amount, year, month, day, is_reduced_rate)
        validated_amount = validate_u64(amount, "amount")
        validated_year, validated_month, validated_day = validate_date_parts(year, month, day)
        result = ConsumptionTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_consumption_tax(
            validated_amount,
            validated_year,
            validated_month,
            validated_day,
            bool_to_c_int(is_reduced_rate),
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        ConsumptionTaxRecord.new(
          tax_amount: result[:tax_amount],
          amount_with_tax: result[:amount_with_tax],
          amount_without_tax: result[:amount_without_tax],
          applied_rate_numer: result[:applied_rate_numer],
          applied_rate_denom: result[:applied_rate_denom],
          is_reduced_rate: c_int_to_bool(result[:is_reduced_rate])
        )
      end

      def calc_stamp_tax(document_code, stated_amount, year, month, day, flags_bitset = 0)
        validated_document_code = validate_u32(document_code, "document_code")
        validated_stated_amount = stated_amount.nil? ? nil : validate_u64(stated_amount, "stated_amount")
        validated_year, validated_month, validated_day = validate_date_parts(year, month, day)
        validated_flags_bitset = validate_u64(flags_bitset, "flags_bitset")
        result = StampTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_stamp_tax(
            validated_document_code,
            validated_stated_amount || 0,
            validated_stated_amount.nil? ? 0 : 1,
            validated_year,
            validated_month,
            validated_day,
            validated_flags_bitset,
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        StampTaxRecord.new(
          tax_amount: result[:tax_amount],
          rule_label: read_fixed_string(result, :rule_label, LABEL_LEN),
          applied_special_rule: begin
            value = read_fixed_string(result, :applied_special_rule, LABEL_LEN)
            value.empty? ? nil : value
          end
        )
      end

      def calc_withholding_tax(
        payment_amount,
        separated_consumption_tax_amount,
        year,
        month,
        day,
        category,
        is_submission_prize
      )
        validated_payment_amount = validate_u64(payment_amount, "payment_amount")
        validated_separated_consumption_tax_amount =
          validate_u64(separated_consumption_tax_amount, "separated_consumption_tax_amount")
        validated_year, validated_month, validated_day = validate_date_parts(year, month, day)
        validated_category = validate_u32(category, "category")
        result = WithholdingTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_withholding_tax(
            validated_payment_amount,
            validated_separated_consumption_tax_amount,
            validated_year,
            validated_month,
            validated_day,
            validated_category,
            bool_to_c_int(is_submission_prize),
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        WithholdingTaxRecord.new(
          gross_payment_amount: result[:gross_payment_amount],
          taxable_payment_amount: result[:taxable_payment_amount],
          tax_amount: result[:tax_amount],
          net_payment_amount: result[:net_payment_amount],
          category: result[:category],
          submission_prize_exempted: c_int_to_bool(result[:submission_prize_exempted]),
          breakdown: read_struct_array(
            result.pointer + WithholdingTaxStruct.offset_of(:breakdown_storage),
            BreakdownStepStruct,
            result[:breakdown_len]
          ).map do |step|
            BreakdownStepRecord.new(
              label: read_fixed_string(step, :label, LABEL_LEN),
              base_amount: step[:base_amount],
              rate_numer: step[:rate_numer],
              rate_denom: step[:rate_denom],
              result: step[:result]
            )
          end
        )
      end

      def call_with_error
        error_buf = FFI::MemoryPointer.new(:char, ERROR_BUF_LEN)
        error_buf.write_string("")
        status = yield(error_buf)
        return if status.zero?

        message = error_buf.read_string
        message = "j-law-c-ffi call failed with status #{status}" if message.empty?
        raise RuntimeError, message
      end

      def bool_to_c_int(value)
        value ? 1 : 0
      end

      def c_int_to_bool(value)
        !value.zero?
      end

      def validate_u64(value, field)
        validate_unsigned_integer(value, field, UINT64_MAX)
      end

      def validate_u32(value, field)
        validate_unsigned_integer(value, field, UINT32_MAX)
      end

      def validate_u16(value, field)
        validate_unsigned_integer(value, field, UINT16_MAX)
      end

      def validate_u8(value, field)
        validate_unsigned_integer(value, field, UINT8_MAX)
      end

      def validate_unsigned_integer(value, field, max_value)
        unless value.is_a?(Integer)
          raise TypeError, "#{field} には Integer を指定してください (got #{value.class})"
        end

        if value.negative?
          raise ArgumentError, "#{field} には 0 以上の値を指定してください"
        end

        if value > max_value
          raise ArgumentError, "#{field} は #{max_value} 以下で指定してください"
        end

        value
      end

      def validate_date_parts(year, month, day)
        validated_year = validate_u16(year, "year")
        validated_month = validate_u8(month, "month")
        validated_day = validate_u8(day, "day")
        Date.new(validated_year, validated_month, validated_day)
        [validated_year, validated_month, validated_day]
      rescue Date::Error => e
        raise ArgumentError,
              "無効な日付です: #{format_date_parts(validated_year, validated_month, validated_day)} (#{e.message})"
      end

      def format_date_parts(year, month, day)
        format("%04d-%02d-%02d", year, month, day)
      end

      def read_struct_array(base_pointer, struct_class, length, max_length: MAX_TIERS)
        safe_length = length.clamp(0, max_length)
        Array.new(safe_length) do |index|
          struct_class.new(base_pointer + (index * struct_class.size))
        end
      end

      def build_income_deduction_input_struct(input)
        spouse = input[:spouse]
        dependent = input.fetch(:dependent, {})
        medical = input[:medical]
        life_insurance = input[:life_insurance]
        donation = input[:donation]
        year, month, day = validate_date_parts(input[:year], input[:month], input[:day])

        struct = IncomeDeductionInputStruct.new
        struct[:total_income_amount] = validate_u64(input[:total_income_amount], "total_income_amount")
        struct[:year] = year
        struct[:month] = month
        struct[:day] = day
        struct[:has_spouse] = bool_to_c_int(!spouse.nil?)
        struct[:spouse_total_income_amount] =
          validate_u64(spouse&.fetch(:spouse_total_income_amount, 0) || 0, "spouse_total_income_amount")
        struct[:spouse_is_same_household] = bool_to_c_int(spouse&.fetch(:is_same_household, false) || false)
        struct[:spouse_is_elderly] = bool_to_c_int(spouse&.fetch(:is_elderly, false) || false)
        struct[:dependent_general_count] = validate_u64(dependent.fetch(:general_count, 0), "dependent.general_count")
        struct[:dependent_specific_count] = validate_u64(dependent.fetch(:specific_count, 0), "dependent.specific_count")
        struct[:dependent_elderly_cohabiting_count] =
          validate_u64(dependent.fetch(:elderly_cohabiting_count, 0), "dependent.elderly_cohabiting_count")
        struct[:dependent_elderly_other_count] =
          validate_u64(dependent.fetch(:elderly_other_count, 0), "dependent.elderly_other_count")
        struct[:social_insurance_premium_paid] =
          validate_u64(input.fetch(:social_insurance_premium_paid, 0), "social_insurance_premium_paid")
        struct[:has_medical] = bool_to_c_int(!medical.nil?)
        struct[:medical_expense_paid] =
          validate_u64(medical&.fetch(:medical_expense_paid, 0) || 0, "medical.medical_expense_paid")
        struct[:medical_reimbursed_amount] =
          validate_u64(medical&.fetch(:reimbursed_amount, 0) || 0, "medical.reimbursed_amount")
        struct[:has_life_insurance] = bool_to_c_int(!life_insurance.nil?)
        struct[:life_new_general_paid_amount] =
          validate_u64(life_insurance&.fetch(:new_general_paid_amount, 0) || 0, "life_insurance.new_general_paid_amount")
        struct[:life_new_individual_pension_paid_amount] =
          validate_u64(life_insurance&.fetch(:new_individual_pension_paid_amount, 0) || 0, "life_insurance.new_individual_pension_paid_amount")
        struct[:life_new_care_medical_paid_amount] =
          validate_u64(life_insurance&.fetch(:new_care_medical_paid_amount, 0) || 0, "life_insurance.new_care_medical_paid_amount")
        struct[:life_old_general_paid_amount] =
          validate_u64(life_insurance&.fetch(:old_general_paid_amount, 0) || 0, "life_insurance.old_general_paid_amount")
        struct[:life_old_individual_pension_paid_amount] =
          validate_u64(life_insurance&.fetch(:old_individual_pension_paid_amount, 0) || 0, "life_insurance.old_individual_pension_paid_amount")
        struct[:has_donation] = bool_to_c_int(!donation.nil?)
        struct[:donation_qualified_amount] =
          validate_u64(donation&.fetch(:qualified_donation_amount, 0) || 0, "donation.qualified_donation_amount")
        struct
      end

      def read_fixed_string(struct, field, length)
        bytes = struct.pointer.get_bytes(struct.class.offset_of(field), length)
        bytes.split("\x00", 2).first.force_encoding("UTF-8")
      end

      private_class_method :call_with_error, :bool_to_c_int, :c_int_to_bool,
                           :validate_u64, :validate_u32, :validate_u16, :validate_u8,
                           :validate_unsigned_integer, :validate_date_parts, :format_date_parts,
                           :read_struct_array, :read_fixed_string,
                           :build_income_deduction_input_struct
    end
  end
end
