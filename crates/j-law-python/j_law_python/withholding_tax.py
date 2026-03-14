"""所得税法第204条第1項に基づく報酬・料金等の源泉徴収税額計算。"""

from __future__ import annotations

import datetime
from enum import IntEnum

from ._c_ffi import BreakdownStepRecord
from ._c_ffi import CFFIError
from ._c_ffi import WithholdingTaxRecord
from ._c_ffi import calc_withholding_tax as _calc_withholding_tax


class WithholdingTaxCategory(IntEnum):
    """源泉徴収カテゴリ。"""

    MANUSCRIPT_AND_LECTURE = 1
    PROFESSIONAL_FEE = 2
    EXCLUSIVE_CONTRACT_FEE = 3

    @classmethod
    def from_value(cls, value: "WithholdingTaxCategory | str | int") -> "WithholdingTaxCategory":
        if isinstance(value, cls):
            return value
        if isinstance(value, str):
            normalized = value.strip().lower()
            mapping = {
                "manuscript_and_lecture": cls.MANUSCRIPT_AND_LECTURE,
                "professional_fee": cls.PROFESSIONAL_FEE,
                "exclusive_contract_fee": cls.EXCLUSIVE_CONTRACT_FEE,
            }
            try:
                return mapping[normalized]
            except KeyError as e:
                raise ValueError(f"unknown withholding tax category: {value}") from e
        try:
            return cls(int(value))
        except ValueError as e:
            raise ValueError(f"unknown withholding tax category: {value}") from e


class WithholdingTaxResult:
    """源泉徴収税額の計算結果。"""

    def __init__(self, r: WithholdingTaxRecord) -> None:
        self.gross_payment_amount: int = r.gross_payment_amount
        self.taxable_payment_amount: int = r.taxable_payment_amount
        self.tax_amount: int = r.tax_amount
        self.net_payment_amount: int = r.net_payment_amount
        self.category: WithholdingTaxCategory = WithholdingTaxCategory(r.category)
        self.submission_prize_exempted: bool = r.submission_prize_exempted
        self.breakdown: list[dict[str, int | str]] = [
            _step_to_dict(step) for step in r.breakdown
        ]

    def __repr__(self) -> str:
        return (
            f"WithholdingTaxResult("
            f"gross_payment_amount={self.gross_payment_amount}, "
            f"taxable_payment_amount={self.taxable_payment_amount}, "
            f"tax_amount={self.tax_amount}, "
            f"net_payment_amount={self.net_payment_amount}, "
            f"category={self.category.name}, "
            f"submission_prize_exempted={self.submission_prize_exempted})"
        )


def _step_to_dict(step: BreakdownStepRecord) -> dict[str, int | str]:
    return {
        "label": step.label,
        "base_amount": step.base_amount,
        "rate_numer": step.rate_numer,
        "rate_denom": step.rate_denom,
        "result": step.result,
    }


def calc_withholding_tax(
    payment_amount: int,
    date: datetime.date,
    category: WithholdingTaxCategory | str | int,
    *,
    is_submission_prize: bool = False,
    separated_consumption_tax_amount: int = 0,
) -> WithholdingTaxResult:
    """報酬・料金等の源泉徴収税額を計算する。

    # 法的根拠
    所得税法 第204条第1項
    国税庁タックスアンサー No.2795 / No.2798 / No.2810
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )

    normalized_category = WithholdingTaxCategory.from_value(category)
    try:
        r = _calc_withholding_tax(
            payment_amount,
            separated_consumption_tax_amount,
            date.year,
            date.month,
            date.day,
            int(normalized_category),
            is_submission_prize,
        )
    except CFFIError as e:
        raise ValueError(str(e)) from e

    return WithholdingTaxResult(r)
