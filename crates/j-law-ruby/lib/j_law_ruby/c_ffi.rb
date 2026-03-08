# frozen_string_literal: true

require "ffi"
require_relative "build_support"

module JLawRuby
  module Internal
    module CFFI
      extend FFI::Library

      FFI_VERSION = 1
      MAX_TIERS = 8
      LABEL_LEN = 64
      ERROR_BUF_LEN = 256
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
      attach_function :j_law_calc_consumption_tax,
                      [:uint64, :uint16, :uint8, :uint8, :int,
                       ConsumptionTaxStruct.by_ref, :pointer, :int],
                      :int
      attach_function :j_law_calc_stamp_tax,
                      [:uint64, :uint16, :uint8, :uint8, :int,
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

      def calc_stamp_tax(contract_amount, year, month, day, is_reduced_rate_applicable)
        result = StampTaxStruct.new

        call_with_error do |error_buf|
          j_law_calc_stamp_tax(
            contract_amount,
            year,
            month,
            day,
            bool_to_c_int(is_reduced_rate_applicable),
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

      def read_struct_array(base_pointer, struct_class, length)
        safe_length = length.clamp(0, MAX_TIERS)
        Array.new(safe_length) do |index|
          struct_class.new(base_pointer + (index * struct_class.size))
        end
      end

      def read_fixed_string(struct, field, length)
        bytes = struct.pointer.get_bytes(struct.class.offset_of(field), length)
        bytes.split("\x00", 2).first.force_encoding("UTF-8")
      end

      private_class_method :call_with_error, :bool_to_c_int, :c_int_to_bool,
                           :read_struct_array, :read_fixed_string
    end
  end
end
