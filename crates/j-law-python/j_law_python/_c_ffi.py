"""ctypes adapter for j-law-c-ffi."""

from __future__ import annotations

import ctypes
from dataclasses import dataclass

from . import build_support

FFI_VERSION = build_support.FFI_VERSION
MAX_TIERS = 8
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
