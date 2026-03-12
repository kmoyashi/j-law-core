"""ctypes adapter for j-law-c-ffi."""

from __future__ import annotations

import ctypes
from dataclasses import dataclass

from . import build_support

FFI_VERSION = build_support.FFI_VERSION
MAX_TIERS = 8
MAX_DEDUCTION_LINES = 8
LABEL_LEN = 64
ERROR_BUF_LEN = 256
U64_MAX = (1 << 64) - 1


class CFFIError(RuntimeError):
    """Raised when the C FFI returns an error."""


class _BreakdownStepStruct(ctypes.Structure):
    _fields_ = [
        ("label", ctypes.c_char * LABEL_LEN),
        ("base_amount", ctypes.c_uint64),
        ("rate_numer", ctypes.c_uint64),
        ("rate_denom", ctypes.c_uint64),
        ("result", ctypes.c_uint64),
    ]


class _BrokerageFeeResultStruct(ctypes.Structure):
    _fields_ = [
        ("total_without_tax", ctypes.c_uint64),
        ("total_with_tax", ctypes.c_uint64),
        ("tax_amount", ctypes.c_uint64),
        ("low_cost_special_applied", ctypes.c_int),
        ("breakdown", _BreakdownStepStruct * MAX_TIERS),
        ("breakdown_len", ctypes.c_int),
    ]


class _SocialInsuranceResultStruct(ctypes.Structure):
    _fields_ = [
        ("health_related_amount", ctypes.c_uint64),
        ("pension_amount", ctypes.c_uint64),
        ("total_amount", ctypes.c_uint64),
        ("health_standard_monthly_remuneration", ctypes.c_uint64),
        ("pension_standard_monthly_remuneration", ctypes.c_uint64),
        ("care_insurance_applied", ctypes.c_int),
        ("breakdown", _BreakdownStepStruct * MAX_TIERS),
        ("breakdown_len", ctypes.c_int),
    ]


class _IncomeTaxStepStruct(ctypes.Structure):
    _fields_ = [
        ("label", ctypes.c_char * LABEL_LEN),
        ("taxable_income", ctypes.c_uint64),
        ("rate_numer", ctypes.c_uint64),
        ("rate_denom", ctypes.c_uint64),
        ("deduction", ctypes.c_uint64),
        ("result", ctypes.c_uint64),
    ]


class _IncomeTaxResultStruct(ctypes.Structure):
    _fields_ = [
        ("base_tax", ctypes.c_uint64),
        ("reconstruction_tax", ctypes.c_uint64),
        ("total_tax", ctypes.c_uint64),
        ("reconstruction_tax_applied", ctypes.c_int),
        ("breakdown", _IncomeTaxStepStruct * MAX_TIERS),
        ("breakdown_len", ctypes.c_int),
    ]


class _IncomeDeductionLineStruct(ctypes.Structure):
    _fields_ = [
        ("kind", ctypes.c_uint32),
        ("label", ctypes.c_char * LABEL_LEN),
        ("amount", ctypes.c_uint64),
    ]


class _IncomeDeductionInputStruct(ctypes.Structure):
    _fields_ = [
        ("total_income_amount", ctypes.c_uint64),
        ("year", ctypes.c_uint16),
        ("month", ctypes.c_uint8),
        ("day", ctypes.c_uint8),
        ("has_spouse", ctypes.c_int),
        ("spouse_total_income_amount", ctypes.c_uint64),
        ("spouse_is_same_household", ctypes.c_int),
        ("spouse_is_elderly", ctypes.c_int),
        ("dependent_general_count", ctypes.c_uint64),
        ("dependent_specific_count", ctypes.c_uint64),
        ("dependent_elderly_cohabiting_count", ctypes.c_uint64),
        ("dependent_elderly_other_count", ctypes.c_uint64),
        ("social_insurance_premium_paid", ctypes.c_uint64),
        ("has_medical", ctypes.c_int),
        ("medical_expense_paid", ctypes.c_uint64),
        ("medical_reimbursed_amount", ctypes.c_uint64),
        ("has_life_insurance", ctypes.c_int),
        ("life_new_general_paid_amount", ctypes.c_uint64),
        ("life_new_individual_pension_paid_amount", ctypes.c_uint64),
        ("life_new_care_medical_paid_amount", ctypes.c_uint64),
        ("life_old_general_paid_amount", ctypes.c_uint64),
        ("life_old_individual_pension_paid_amount", ctypes.c_uint64),
        ("has_donation", ctypes.c_int),
        ("donation_qualified_amount", ctypes.c_uint64),
    ]


class _IncomeDeductionResultStruct(ctypes.Structure):
    _fields_ = [
        ("total_income_amount", ctypes.c_uint64),
        ("total_deductions", ctypes.c_uint64),
        ("taxable_income_before_truncation", ctypes.c_uint64),
        ("taxable_income", ctypes.c_uint64),
        ("breakdown", _IncomeDeductionLineStruct * MAX_DEDUCTION_LINES),
        ("breakdown_len", ctypes.c_int),
    ]


class _IncomeTaxAssessmentResultStruct(ctypes.Structure):
    _fields_ = [
        ("total_income_amount", ctypes.c_uint64),
        ("total_deductions", ctypes.c_uint64),
        ("taxable_income_before_truncation", ctypes.c_uint64),
        ("taxable_income", ctypes.c_uint64),
        ("base_tax", ctypes.c_uint64),
        ("reconstruction_tax", ctypes.c_uint64),
        ("total_tax", ctypes.c_uint64),
        ("reconstruction_tax_applied", ctypes.c_int),
        ("deduction_breakdown", _IncomeDeductionLineStruct * MAX_DEDUCTION_LINES),
        ("deduction_breakdown_len", ctypes.c_int),
        ("tax_breakdown", _IncomeTaxStepStruct * MAX_TIERS),
        ("tax_breakdown_len", ctypes.c_int),
    ]


class _ConsumptionTaxResultStruct(ctypes.Structure):
    _fields_ = [
        ("tax_amount", ctypes.c_uint64),
        ("amount_with_tax", ctypes.c_uint64),
        ("amount_without_tax", ctypes.c_uint64),
        ("applied_rate_numer", ctypes.c_uint64),
        ("applied_rate_denom", ctypes.c_uint64),
        ("is_reduced_rate", ctypes.c_int),
    ]


class _StampTaxResultStruct(ctypes.Structure):
    _fields_ = [
        ("tax_amount", ctypes.c_uint64),
        ("bracket_label", ctypes.c_char * LABEL_LEN),
        ("reduced_rate_applied", ctypes.c_int),
    ]


@dataclass(frozen=True)
class BreakdownStepRecord:
    label: str
    base_amount: int
    rate_numer: int
    rate_denom: int
    result: int


@dataclass(frozen=True)
class BrokerageFeeRecord:
    total_without_tax: int
    total_with_tax: int
    tax_amount: int
    low_cost_special_applied: bool
    breakdown: list[BreakdownStepRecord]


@dataclass(frozen=True)
class SocialInsuranceRecord:
    health_related_amount: int
    pension_amount: int
    total_amount: int
    health_standard_monthly_remuneration: int
    pension_standard_monthly_remuneration: int
    care_insurance_applied: bool
    breakdown: list[BreakdownStepRecord]


@dataclass(frozen=True)
class IncomeTaxStepRecord:
    label: str
    taxable_income: int
    rate_numer: int
    rate_denom: int
    deduction: int
    result: int


@dataclass(frozen=True)
class IncomeTaxRecord:
    base_tax: int
    reconstruction_tax: int
    total_tax: int
    reconstruction_tax_applied: bool
    breakdown: list[IncomeTaxStepRecord]


@dataclass(frozen=True)
class IncomeDeductionInputRecord:
    total_income_amount: int
    year: int
    month: int
    day: int
    has_spouse: bool = False
    spouse_total_income_amount: int = 0
    spouse_is_same_household: bool = False
    spouse_is_elderly: bool = False
    dependent_general_count: int = 0
    dependent_specific_count: int = 0
    dependent_elderly_cohabiting_count: int = 0
    dependent_elderly_other_count: int = 0
    social_insurance_premium_paid: int = 0
    has_medical: bool = False
    medical_expense_paid: int = 0
    medical_reimbursed_amount: int = 0
    has_life_insurance: bool = False
    life_new_general_paid_amount: int = 0
    life_new_individual_pension_paid_amount: int = 0
    life_new_care_medical_paid_amount: int = 0
    life_old_general_paid_amount: int = 0
    life_old_individual_pension_paid_amount: int = 0
    has_donation: bool = False
    donation_qualified_amount: int = 0


@dataclass(frozen=True)
class IncomeDeductionLineRecord:
    kind: int
    label: str
    amount: int


@dataclass(frozen=True)
class IncomeDeductionRecord:
    total_income_amount: int
    total_deductions: int
    taxable_income_before_truncation: int
    taxable_income: int
    breakdown: list[IncomeDeductionLineRecord]


@dataclass(frozen=True)
class IncomeTaxAssessmentRecord:
    deductions: IncomeDeductionRecord
    tax: IncomeTaxRecord


@dataclass(frozen=True)
class ConsumptionTaxRecord:
    tax_amount: int
    amount_with_tax: int
    amount_without_tax: int
    applied_rate_numer: int
    applied_rate_denom: int
    is_reduced_rate: bool


@dataclass(frozen=True)
class StampTaxRecord:
    tax_amount: int
    bracket_label: str
    reduced_rate_applied: bool


def _decode_fixed_string(raw: ctypes.Array[ctypes.c_char]) -> str:
    return bytes(raw).split(b"\0", 1)[0].decode("utf-8")


def _bool_to_c_int(value: bool) -> int:
    return 1 if value else 0


def _validate_u64(value: int, field_name: str) -> int:
    if value < 0 or value > U64_MAX:
        raise CFFIError(f"{field_name} must be between 0 and {U64_MAX}")
    return value


def _read_error(buffer: ctypes.Array[ctypes.c_char]) -> str:
    message = buffer.value.decode("utf-8")
    if message:
        return message
    return "j-law-c-ffi returned an unknown error"


def _read_breakdown(
    steps: ctypes.Array[_BreakdownStepStruct],
    length: int,
) -> list[BreakdownStepRecord]:
    safe_length = max(0, min(length, MAX_TIERS))
    return [
        BreakdownStepRecord(
            label=_decode_fixed_string(step.label),
            base_amount=int(step.base_amount),
            rate_numer=int(step.rate_numer),
            rate_denom=int(step.rate_denom),
            result=int(step.result),
        )
        for step in steps[:safe_length]
    ]


def _read_income_tax_breakdown(
    steps: ctypes.Array[_IncomeTaxStepStruct],
    length: int,
) -> list[IncomeTaxStepRecord]:
    safe_length = max(0, min(length, MAX_TIERS))
    return [
        IncomeTaxStepRecord(
            label=_decode_fixed_string(step.label),
            taxable_income=int(step.taxable_income),
            rate_numer=int(step.rate_numer),
            rate_denom=int(step.rate_denom),
            deduction=int(step.deduction),
            result=int(step.result),
        )
        for step in steps[:safe_length]
    ]


def _read_income_deduction_breakdown(
    steps: ctypes.Array[_IncomeDeductionLineStruct],
    length: int,
) -> list[IncomeDeductionLineRecord]:
    safe_length = max(0, min(length, MAX_DEDUCTION_LINES))
    return [
        IncomeDeductionLineRecord(
            kind=int(step.kind),
            label=_decode_fixed_string(step.label),
            amount=int(step.amount),
        )
        for step in steps[:safe_length]
    ]


def _build_income_deduction_input_struct(
    record: IncomeDeductionInputRecord,
) -> _IncomeDeductionInputStruct:
    struct = _IncomeDeductionInputStruct()
    struct.total_income_amount = _validate_u64(
        record.total_income_amount,
        "total_income_amount",
    )
    struct.year = record.year
    struct.month = record.month
    struct.day = record.day
    struct.has_spouse = _bool_to_c_int(record.has_spouse)
    struct.spouse_total_income_amount = _validate_u64(
        record.spouse_total_income_amount,
        "spouse_total_income_amount",
    )
    struct.spouse_is_same_household = _bool_to_c_int(
        record.spouse_is_same_household
    )
    struct.spouse_is_elderly = _bool_to_c_int(record.spouse_is_elderly)
    struct.dependent_general_count = _validate_u64(
        record.dependent_general_count,
        "dependent_general_count",
    )
    struct.dependent_specific_count = _validate_u64(
        record.dependent_specific_count,
        "dependent_specific_count",
    )
    struct.dependent_elderly_cohabiting_count = _validate_u64(
        record.dependent_elderly_cohabiting_count,
        "dependent_elderly_cohabiting_count",
    )
    struct.dependent_elderly_other_count = _validate_u64(
        record.dependent_elderly_other_count,
        "dependent_elderly_other_count",
    )
    struct.social_insurance_premium_paid = _validate_u64(
        record.social_insurance_premium_paid,
        "social_insurance_premium_paid",
    )
    struct.has_medical = _bool_to_c_int(record.has_medical)
    struct.medical_expense_paid = _validate_u64(
        record.medical_expense_paid,
        "medical_expense_paid",
    )
    struct.medical_reimbursed_amount = _validate_u64(
        record.medical_reimbursed_amount,
        "medical_reimbursed_amount",
    )
    struct.has_life_insurance = _bool_to_c_int(record.has_life_insurance)
    struct.life_new_general_paid_amount = _validate_u64(
        record.life_new_general_paid_amount,
        "life_new_general_paid_amount",
    )
    struct.life_new_individual_pension_paid_amount = _validate_u64(
        record.life_new_individual_pension_paid_amount,
        "life_new_individual_pension_paid_amount",
    )
    struct.life_new_care_medical_paid_amount = _validate_u64(
        record.life_new_care_medical_paid_amount,
        "life_new_care_medical_paid_amount",
    )
    struct.life_old_general_paid_amount = _validate_u64(
        record.life_old_general_paid_amount,
        "life_old_general_paid_amount",
    )
    struct.life_old_individual_pension_paid_amount = _validate_u64(
        record.life_old_individual_pension_paid_amount,
        "life_old_individual_pension_paid_amount",
    )
    struct.has_donation = _bool_to_c_int(record.has_donation)
    struct.donation_qualified_amount = _validate_u64(
        record.donation_qualified_amount,
        "donation_qualified_amount",
    )
    return struct


LIBRARY_PATH = build_support.resolve_shared_library_path()
if LIBRARY_PATH is None:
    raise ImportError(
        "j-law-c-ffi shared library was not found. "
        "Set JLAW_PYTHON_C_FFI_LIB or build it with `cargo build -p j-law-c-ffi`."
    )

_LIB = ctypes.CDLL(str(LIBRARY_PATH))
_LIB.j_law_c_ffi_version.argtypes = []
_LIB.j_law_c_ffi_version.restype = ctypes.c_uint32
_LIB.j_law_calc_brokerage_fee.argtypes = [
    ctypes.c_uint64,
    ctypes.c_uint16,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.POINTER(_BrokerageFeeResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_brokerage_fee.restype = ctypes.c_int
_LIB.j_law_calc_social_insurance.argtypes = [
    ctypes.c_uint64,
    ctypes.c_uint16,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_int,
    ctypes.POINTER(_SocialInsuranceResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_social_insurance.restype = ctypes.c_int
_LIB.j_law_calc_income_tax.argtypes = [
    ctypes.c_uint64,
    ctypes.c_uint16,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_int,
    ctypes.POINTER(_IncomeTaxResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_income_tax.restype = ctypes.c_int
_LIB.j_law_calc_income_deductions.argtypes = [
    ctypes.POINTER(_IncomeDeductionInputStruct),
    ctypes.POINTER(_IncomeDeductionResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_income_deductions.restype = ctypes.c_int
_LIB.j_law_calc_income_tax_assessment.argtypes = [
    ctypes.POINTER(_IncomeDeductionInputStruct),
    ctypes.c_int,
    ctypes.POINTER(_IncomeTaxAssessmentResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_income_tax_assessment.restype = ctypes.c_int
_LIB.j_law_calc_consumption_tax.argtypes = [
    ctypes.c_uint64,
    ctypes.c_uint16,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_int,
    ctypes.POINTER(_ConsumptionTaxResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_consumption_tax.restype = ctypes.c_int
_LIB.j_law_calc_stamp_tax.argtypes = [
    ctypes.c_uint64,
    ctypes.c_uint16,
    ctypes.c_uint8,
    ctypes.c_uint8,
    ctypes.c_int,
    ctypes.POINTER(_StampTaxResultStruct),
    ctypes.POINTER(ctypes.c_char),
    ctypes.c_int,
]
_LIB.j_law_calc_stamp_tax.restype = ctypes.c_int

_ACTUAL_FFI_VERSION = _LIB.j_law_c_ffi_version()
if _ACTUAL_FFI_VERSION != FFI_VERSION:
    raise ImportError(
        "j-law-c-ffi FFI version mismatch: "
        f"expected {FFI_VERSION}, got {_ACTUAL_FFI_VERSION}"
    )


def ffi_version() -> int:
    return int(_ACTUAL_FFI_VERSION)


def library_path() -> str:
    return str(LIBRARY_PATH)


def calc_brokerage_fee(
    price: int,
    year: int,
    month: int,
    day: int,
    is_low_cost_vacant_house: bool,
    is_seller: bool,
) -> BrokerageFeeRecord:
    checked_price = _validate_u64(price, "price")
    result = _BrokerageFeeResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_brokerage_fee(
        ctypes.c_uint64(checked_price),
        ctypes.c_uint16(year),
        ctypes.c_uint8(month),
        ctypes.c_uint8(day),
        ctypes.c_int(_bool_to_c_int(is_low_cost_vacant_house)),
        ctypes.c_int(_bool_to_c_int(is_seller)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return BrokerageFeeRecord(
        total_without_tax=int(result.total_without_tax),
        total_with_tax=int(result.total_with_tax),
        tax_amount=int(result.tax_amount),
        low_cost_special_applied=bool(result.low_cost_special_applied),
        breakdown=_read_breakdown(result.breakdown, int(result.breakdown_len)),
    )


def calc_social_insurance(
    standard_monthly_remuneration: int,
    year: int,
    month: int,
    day: int,
    prefecture_code: int,
    is_care_insurance_applicable: bool,
) -> SocialInsuranceRecord:
    checked_amount = _validate_u64(
        standard_monthly_remuneration,
        "standard_monthly_remuneration",
    )
    result = _SocialInsuranceResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_social_insurance(
        ctypes.c_uint64(checked_amount),
        ctypes.c_uint16(year),
        ctypes.c_uint8(month),
        ctypes.c_uint8(day),
        ctypes.c_uint8(prefecture_code),
        ctypes.c_int(_bool_to_c_int(is_care_insurance_applicable)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return SocialInsuranceRecord(
        health_related_amount=int(result.health_related_amount),
        pension_amount=int(result.pension_amount),
        total_amount=int(result.total_amount),
        health_standard_monthly_remuneration=int(
            result.health_standard_monthly_remuneration
        ),
        pension_standard_monthly_remuneration=int(
            result.pension_standard_monthly_remuneration
        ),
        care_insurance_applied=bool(result.care_insurance_applied),
        breakdown=_read_breakdown(result.breakdown, int(result.breakdown_len)),
    )


def calc_income_tax(
    taxable_income: int,
    year: int,
    month: int,
    day: int,
    apply_reconstruction_tax: bool,
) -> IncomeTaxRecord:
    checked_taxable_income = _validate_u64(taxable_income, "taxable_income")
    result = _IncomeTaxResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_income_tax(
        ctypes.c_uint64(checked_taxable_income),
        ctypes.c_uint16(year),
        ctypes.c_uint8(month),
        ctypes.c_uint8(day),
        ctypes.c_int(_bool_to_c_int(apply_reconstruction_tax)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return IncomeTaxRecord(
        base_tax=int(result.base_tax),
        reconstruction_tax=int(result.reconstruction_tax),
        total_tax=int(result.total_tax),
        reconstruction_tax_applied=bool(result.reconstruction_tax_applied),
        breakdown=_read_income_tax_breakdown(
            result.breakdown,
            int(result.breakdown_len),
        ),
    )


def calc_income_deductions(
    record: IncomeDeductionInputRecord,
) -> IncomeDeductionRecord:
    input_struct = _build_income_deduction_input_struct(record)
    result = _IncomeDeductionResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_income_deductions(
        ctypes.byref(input_struct),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return IncomeDeductionRecord(
        total_income_amount=int(result.total_income_amount),
        total_deductions=int(result.total_deductions),
        taxable_income_before_truncation=int(result.taxable_income_before_truncation),
        taxable_income=int(result.taxable_income),
        breakdown=_read_income_deduction_breakdown(
            result.breakdown,
            int(result.breakdown_len),
        ),
    )


def calc_income_tax_assessment(
    record: IncomeDeductionInputRecord,
    apply_reconstruction_tax: bool,
) -> IncomeTaxAssessmentRecord:
    input_struct = _build_income_deduction_input_struct(record)
    result = _IncomeTaxAssessmentResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_income_tax_assessment(
        ctypes.byref(input_struct),
        ctypes.c_int(_bool_to_c_int(apply_reconstruction_tax)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    deductions = IncomeDeductionRecord(
        total_income_amount=int(result.total_income_amount),
        total_deductions=int(result.total_deductions),
        taxable_income_before_truncation=int(result.taxable_income_before_truncation),
        taxable_income=int(result.taxable_income),
        breakdown=_read_income_deduction_breakdown(
            result.deduction_breakdown,
            int(result.deduction_breakdown_len),
        ),
    )
    tax = IncomeTaxRecord(
        base_tax=int(result.base_tax),
        reconstruction_tax=int(result.reconstruction_tax),
        total_tax=int(result.total_tax),
        reconstruction_tax_applied=bool(result.reconstruction_tax_applied),
        breakdown=_read_income_tax_breakdown(
            result.tax_breakdown,
            int(result.tax_breakdown_len),
        ),
    )
    return IncomeTaxAssessmentRecord(deductions=deductions, tax=tax)


def calc_consumption_tax(
    amount: int,
    year: int,
    month: int,
    day: int,
    is_reduced_rate: bool,
) -> ConsumptionTaxRecord:
    checked_amount = _validate_u64(amount, "amount")
    result = _ConsumptionTaxResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_consumption_tax(
        ctypes.c_uint64(checked_amount),
        ctypes.c_uint16(year),
        ctypes.c_uint8(month),
        ctypes.c_uint8(day),
        ctypes.c_int(_bool_to_c_int(is_reduced_rate)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return ConsumptionTaxRecord(
        tax_amount=int(result.tax_amount),
        amount_with_tax=int(result.amount_with_tax),
        amount_without_tax=int(result.amount_without_tax),
        applied_rate_numer=int(result.applied_rate_numer),
        applied_rate_denom=int(result.applied_rate_denom),
        is_reduced_rate=bool(result.is_reduced_rate),
    )


def calc_stamp_tax(
    contract_amount: int,
    year: int,
    month: int,
    day: int,
    is_reduced_rate_applicable: bool,
) -> StampTaxRecord:
    checked_contract_amount = _validate_u64(contract_amount, "contract_amount")
    result = _StampTaxResultStruct()
    error_buffer = ctypes.create_string_buffer(ERROR_BUF_LEN)
    status = _LIB.j_law_calc_stamp_tax(
        ctypes.c_uint64(checked_contract_amount),
        ctypes.c_uint16(year),
        ctypes.c_uint8(month),
        ctypes.c_uint8(day),
        ctypes.c_int(_bool_to_c_int(is_reduced_rate_applicable)),
        ctypes.byref(result),
        error_buffer,
        ERROR_BUF_LEN,
    )
    if status != 0:
        raise CFFIError(_read_error(error_buffer))

    return StampTaxRecord(
        tax_amount=int(result.tax_amount),
        bracket_label=_decode_fixed_string(result.bracket_label),
        reduced_rate_applied=bool(result.reduced_rate_applied),
    )
