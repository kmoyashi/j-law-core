"""印紙税法別表第一に基づく印紙税額計算（Python ラッパー）。"""

from __future__ import annotations

import datetime

from ._c_ffi import CFFIError
from ._c_ffi import StampTaxRecord
from ._c_ffi import calc_stamp_tax as _calc_stamp_tax


class StampTaxResult:
    """印紙税の計算結果。

    Attributes:
        tax_amount (int): 印紙税額（円）
        bracket_label (str): 適用されたブラケットの表示名
        reduced_rate_applied (bool): 軽減税率が適用されたか
    """

    def __init__(self, r: StampTaxRecord) -> None:
        self.tax_amount: int = r.tax_amount
        self.bracket_label: str = r.bracket_label
        self.reduced_rate_applied: bool = r.reduced_rate_applied

    def __repr__(self) -> str:
        return (
            f"StampTaxResult("
            f"tax_amount={self.tax_amount}, "
            f"bracket_label={self.bracket_label!r}, "
            f"reduced_rate_applied={self.reduced_rate_applied})"
        )


def calc_stamp_tax(
    contract_amount: int,
    date: datetime.date,
    is_reduced_rate_applicable: bool = False,
) -> StampTaxResult:
    """印紙税法 別表第一に基づく印紙税額を計算する。

    # 法的根拠
    印紙税法 別表第一 第1号文書（不動産の譲渡に関する契約書）
    租税特別措置法 第91条（軽減措置）

    Args:
        contract_amount (int): 契約金額（円）
        date (datetime.date): 契約書作成日
        is_reduced_rate_applicable (bool): 軽減税率適用フラグ（デフォルト: False）
            WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任。

    Returns:
        StampTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        r = _calc_stamp_tax(
            contract_amount,
            date.year,
            date.month,
            date.day,
            is_reduced_rate_applicable,
        )
    except CFFIError as e:
        raise ValueError(str(e)) from e
    return StampTaxResult(r)
