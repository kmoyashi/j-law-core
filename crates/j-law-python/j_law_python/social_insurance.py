"""協会けんぽ一般被保険者の月額社会保険料計算（Python ラッパー）。"""

from __future__ import annotations

import datetime

from ._c_ffi import CFFIError
from ._c_ffi import SocialInsuranceRecord
from ._c_ffi import calc_social_insurance as _calc_social_insurance


class SocialInsuranceResult:
    """社会保険料の計算結果。"""

    def __init__(self, record: SocialInsuranceRecord) -> None:
        self.health_related_amount: int = record.health_related_amount
        self.pension_amount: int = record.pension_amount
        self.total_amount: int = record.total_amount
        self.health_standard_monthly_remuneration: int = (
            record.health_standard_monthly_remuneration
        )
        self.pension_standard_monthly_remuneration: int = (
            record.pension_standard_monthly_remuneration
        )
        self.care_insurance_applied: bool = record.care_insurance_applied
        self.breakdown: list[dict[str, int | str]] = [
            {
                "label": step.label,
                "base_amount": step.base_amount,
                "rate_numer": step.rate_numer,
                "rate_denom": step.rate_denom,
                "result": step.result,
            }
            for step in record.breakdown
        ]

    def __repr__(self) -> str:
        return (
            "SocialInsuranceResult("
            f"health_related_amount={self.health_related_amount}, "
            f"pension_amount={self.pension_amount}, "
            f"total_amount={self.total_amount}, "
            f"care_insurance_applied={self.care_insurance_applied})"
        )


def calc_social_insurance(
    standard_monthly_remuneration: int,
    date: datetime.date,
    prefecture_code: int,
    *,
    is_care_insurance_applicable: bool = False,
) -> SocialInsuranceResult:
    """協会けんぽ一般被保険者の月額社会保険料本人負担分を計算する。"""

    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )

    try:
        record = _calc_social_insurance(
            standard_monthly_remuneration,
            date.year,
            date.month,
            date.day,
            prefecture_code,
            is_care_insurance_applicable,
        )
    except CFFIError as exc:
        raise ValueError(str(exc)) from exc
    return SocialInsuranceResult(record)
