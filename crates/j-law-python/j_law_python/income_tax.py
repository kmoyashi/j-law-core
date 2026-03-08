"""所得税法第89条に基づく所得税計算（Python ラッパー）。"""

from __future__ import annotations

import datetime
from typing import List

from ._cgo import CgoError
from ._cgo import IncomeTaxRecord
from ._cgo import IncomeTaxStepRecord
from ._cgo import calc_income_tax as _calc_income_tax


class IncomeTaxStep:
    """所得税速算表の1ブラケット分の内訳。

    Attributes:
        label (str): ブラケットの表示名
        taxable_income (int): 課税所得金額（円）
        rate_numer (int): 適用税率の分子
        rate_denom (int): 適用税率の分母
        deduction (int): 速算表の控除額（円）
        result (int): 算出税額（円）
    """

    def __init__(self, r: IncomeTaxStepRecord) -> None:
        self.label: str = r.label
        self.taxable_income: int = r.taxable_income
        self.rate_numer: int = r.rate_numer
        self.rate_denom: int = r.rate_denom
        self.deduction: int = r.deduction
        self.result: int = r.result

    def __repr__(self) -> str:
        return (
            f"IncomeTaxStep("
            f"label={self.label!r}, "
            f"taxable_income={self.taxable_income}, "
            f"rate={self.rate_numer}/{self.rate_denom}, "
            f"deduction={self.deduction}, "
            f"result={self.result})"
        )


class IncomeTaxResult:
    """所得税の計算結果。

    Attributes:
        base_tax (int): 基準所得税額（円）
        reconstruction_tax (int): 復興特別所得税額（円）
        total_tax (int): 申告納税額（円・100円未満切り捨て）
        reconstruction_tax_applied (bool): 復興特別所得税が適用されたか
        breakdown (list[IncomeTaxStep]): 計算内訳
    """

    def __init__(self, r: IncomeTaxRecord) -> None:
        self.base_tax: int = r.base_tax
        self.reconstruction_tax: int = r.reconstruction_tax
        self.total_tax: int = r.total_tax
        self.reconstruction_tax_applied: bool = r.reconstruction_tax_applied
        self.breakdown: List[IncomeTaxStep] = [IncomeTaxStep(s) for s in r.breakdown]

    def __repr__(self) -> str:
        return (
            f"IncomeTaxResult("
            f"base_tax={self.base_tax}, "
            f"reconstruction_tax={self.reconstruction_tax}, "
            f"total_tax={self.total_tax}, "
            f"reconstruction_tax_applied={self.reconstruction_tax_applied})"
        )


def calc_income_tax(
    taxable_income: int,
    date: datetime.date,
    apply_reconstruction_tax: bool = True,
) -> IncomeTaxResult:
    """所得税法第89条に基づく所得税額を計算する。

    # 法的根拠
    所得税法 第89条第1項 / 復興財源確保法 第13条

    Args:
        taxable_income (int): 課税所得金額（円・1,000円未満切り捨て済み）
        date (datetime.date): 基準日
        apply_reconstruction_tax (bool): 復興特別所得税を適用するか（デフォルト: True）

    Returns:
        IncomeTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        r = _calc_income_tax(
            taxable_income,
            date.year,
            date.month,
            date.day,
            apply_reconstruction_tax,
        )
    except CgoError as e:
        raise ValueError(str(e)) from e
    return IncomeTaxResult(r)
