# frozen_string_literal: true

require "ffi"
require_relative "build_support"

module JLawRuby
  module Internal
    module CFFI
      extend FFI::Library

      FFI_VERSION = 3
      MAX_TIERS = 8
      MAX_DEDUCTION_LINES = 8
      LABEL_LEN = 64
      ERROR_BUF_LEN = 256
      STAMP_TAX_DOCUMENT_KIND_REAL_ESTATE_TRANSFER = 0
      STAMP_TAX_DOCUMENT_KIND_CONSTRUCTION_CONTRACT = 1
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
        :tax_amount, :bracket_label, :reduced_rate_applied,
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
               :bracket_label, [:char, LABEL_LEN],
               :reduced_rate_applied, :int
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
                      [:uint64, :uint16, :uint8, :uint8, :int,
                       StampTaxStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_stamp_tax_with_document_kind,
                      [:uint64, :uint16, :uint8, :uint8, :int, :int,
                       StampTaxStruct.by_ref, :pointer, :int],
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
        result = BrokerageFeeStruct.new

        call_with_error do |error_buf|
          j_law_calc_brokerage_fee(
            price,
            year,
            month,
            day,
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
        result = IncomeTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_income_tax(
            taxable_income,
            year,
            month,
            day,
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
        result = ConsumptionTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_consumption_tax(
            amount,
            year,
            month,
            day,
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

      def calc_stamp_tax(contract_amount, year, month, day, is_reduced_rate_applicable,
                         document_kind = STAMP_TAX_DOCUMENT_KIND_REAL_ESTATE_TRANSFER)
        result = StampTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_stamp_tax_with_document_kind(
            contract_amount,
            year,
            month,
            day,
            bool_to_c_int(is_reduced_rate_applicable),
            document_kind,
            result,
            error_buf,
            ERROR_BUF_LEN
          )
        end

        StampTaxRecord.new(
          tax_amount: result[:tax_amount],
          bracket_label: read_fixed_string(result, :bracket_label, LABEL_LEN),
          reduced_rate_applied: c_int_to_bool(result[:reduced_rate_applied])
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

        struct = IncomeDeductionInputStruct.new
        struct[:total_income_amount] = input[:total_income_amount]
        struct[:year] = input[:year]
        struct[:month] = input[:month]
        struct[:day] = input[:day]
        struct[:has_spouse] = bool_to_c_int(!spouse.nil?)
        struct[:spouse_total_income_amount] = spouse&.fetch(:spouse_total_income_amount, 0) || 0
        struct[:spouse_is_same_household] = bool_to_c_int(spouse&.fetch(:is_same_household, false) || false)
        struct[:spouse_is_elderly] = bool_to_c_int(spouse&.fetch(:is_elderly, false) || false)
        struct[:dependent_general_count] = dependent.fetch(:general_count, 0)
        struct[:dependent_specific_count] = dependent.fetch(:specific_count, 0)
        struct[:dependent_elderly_cohabiting_count] = dependent.fetch(:elderly_cohabiting_count, 0)
        struct[:dependent_elderly_other_count] = dependent.fetch(:elderly_other_count, 0)
        struct[:social_insurance_premium_paid] = input.fetch(:social_insurance_premium_paid, 0)
        struct[:has_medical] = bool_to_c_int(!medical.nil?)
        struct[:medical_expense_paid] = medical&.fetch(:medical_expense_paid, 0) || 0
        struct[:medical_reimbursed_amount] = medical&.fetch(:reimbursed_amount, 0) || 0
        struct[:has_life_insurance] = bool_to_c_int(!life_insurance.nil?)
        struct[:life_new_general_paid_amount] = life_insurance&.fetch(:new_general_paid_amount, 0) || 0
        struct[:life_new_individual_pension_paid_amount] = life_insurance&.fetch(:new_individual_pension_paid_amount, 0) || 0
        struct[:life_new_care_medical_paid_amount] = life_insurance&.fetch(:new_care_medical_paid_amount, 0) || 0
        struct[:life_old_general_paid_amount] = life_insurance&.fetch(:old_general_paid_amount, 0) || 0
        struct[:life_old_individual_pension_paid_amount] = life_insurance&.fetch(:old_individual_pension_paid_amount, 0) || 0
        struct[:has_donation] = bool_to_c_int(!donation.nil?)
        struct[:donation_qualified_amount] = donation&.fetch(:qualified_donation_amount, 0) || 0
        struct
      end

      def read_fixed_string(struct, field, length)
        bytes = struct.pointer.get_bytes(struct.class.offset_of(field), length)
        bytes.split("\x00", 2).first.force_encoding("UTF-8")
      end

      private_class_method :call_with_error, :bool_to_c_int, :c_int_to_bool,
                           :read_struct_array, :read_fixed_string,
                           :build_income_deduction_input_struct
    end
  end
end
