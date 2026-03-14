"""印紙税法別表第一に基づく印紙税額計算（Python ラッパー）。"""

from __future__ import annotations

import datetime
from collections.abc import Iterable

from ._c_ffi import CFFIError
from ._c_ffi import StampTaxRecord
from ._c_ffi import calc_stamp_tax as _calc_stamp_tax

_DOCUMENT_CODE_MAP = {
    "article1_real_estate_transfer": 1,
    "article1_other_transfer": 2,
    "article1_land_lease_or_surface_right": 3,
    "article1_consumption_loan": 4,
    "article1_transportation": 5,
    "article2_construction_work": 6,
    "article2_general_contract": 7,
    "article3_bill_amount_table": 8,
    "article3_bill_special_flat_200": 9,
    "article4_security_certificate": 10,
    "article5_merger_or_split": 11,
    "article6_articles_of_incorporation": 12,
    "article7_continuing_transaction_basic": 13,
    "article8_deposit_certificate": 14,
    "article9_transport_certificate": 15,
    "article10_insurance_certificate": 16,
    "article11_letter_of_credit": 17,
    "article12_trust_contract": 18,
    "article13_debt_guarantee": 19,
    "article14_deposit_contract": 20,
    "article15_assignment_or_assumption": 21,
    "article16_dividend_receipt": 22,
    "article17_sales_receipt": 23,
    "article17_other_receipt": 24,
    "article18_passbook": 25,
    "article19_misc_passbook": 26,
    "article20_seal_book": 27,
}

_FLAG_BIT_MAP = {
    "article3_copy_or_transcript_exempt": 1 << 0,
    "article4_specified_issuer_exempt": 1 << 1,
    "article4_restricted_beneficiary_certificate_exempt": 1 << 2,
    "article6_notary_copy_exempt": 1 << 3,
    "article8_small_deposit_exempt": 1 << 4,
    "article13_identity_guarantee_exempt": 1 << 5,
    "article17_non_business_exempt": 1 << 6,
    "article17_appended_receipt_exempt": 1 << 7,
    "article18_specified_financial_institution_exempt": 1 << 8,
    "article18_income_tax_exempt_passbook": 1 << 9,
    "article18_tax_reserve_deposit_passbook": 1 << 10,
}


class StampTaxResult:
    """印紙税の計算結果。"""

    def __init__(self, r: StampTaxRecord) -> None:
        self.tax_amount: int = r.tax_amount
        self.rule_label: str = r.rule_label
        self.applied_special_rule: str | None = r.applied_special_rule

    def __repr__(self) -> str:
        return (
            "StampTaxResult("
            f"tax_amount={self.tax_amount}, "
            f"rule_label={self.rule_label!r}, "
            f"applied_special_rule={self.applied_special_rule!r})"
        )


def calc_stamp_tax(
    document_code: str,
    stated_amount: int | None,
    date: datetime.date,
    flags: Iterable[str] | None = None,
) -> StampTaxResult:
    """印紙税法 別表第一に基づく印紙税額を計算する。"""

    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    if not isinstance(document_code, str):
        raise TypeError(
            "document_code には str を指定してください "
            f"(got {type(document_code).__name__})"
        )
    if stated_amount is not None and not isinstance(stated_amount, int):
        raise TypeError(
            "stated_amount には int または None を指定してください "
            f"(got {type(stated_amount).__name__})"
        )

    try:
        document_code_value = _DOCUMENT_CODE_MAP[document_code]
    except KeyError as e:
        raise ValueError(f"unsupported stamp tax document_code: {document_code}") from e

    bitset = 0
    if flags is not None:
        for flag in flags:
            if not isinstance(flag, str):
                raise TypeError(
                    f"flags には str の iterable を指定してください (got {type(flag).__name__})"
                )
            try:
                bitset |= _FLAG_BIT_MAP[flag]
            except KeyError as e:
                raise ValueError(f"unsupported stamp tax flag: {flag}") from e

    try:
        r = _calc_stamp_tax(
            document_code_value,
            stated_amount,
            date.year,
            date.month,
            date.day,
            bitset,
        )
    except CFFIError as e:
        raise ValueError(str(e)) from e
    return StampTaxResult(r)
