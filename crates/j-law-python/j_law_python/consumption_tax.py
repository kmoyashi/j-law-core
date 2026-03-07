"""消費税法第29条に基づく消費税額計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime

import j_law_uniffi


class ConsumptionTaxResult:
    """消費税の計算結果。

    Attributes:
        tax_amount (int): 消費税額（円）
        amount_with_tax (int): 税込金額（円）
        amount_without_tax (int): 税抜金額（円）
        applied_rate_numer (int): 適用税率の分子
        applied_rate_denom (int): 適用税率の分母
        is_reduced_rate (bool): 軽減税率が適用されたか
    """

    def __init__(self, r: j_law_uniffi.UniConsumptionTaxResult) -> None:
        self.tax_amount: int = r.tax_amount
        self.amount_with_tax: int = r.amount_with_tax
        self.amount_without_tax: int = r.amount_without_tax
        self.applied_rate_numer: int = r.applied_rate_numer
        self.applied_rate_denom: int = r.applied_rate_denom
        self.is_reduced_rate: bool = r.is_reduced_rate

    def __repr__(self) -> str:
        return (
            f"ConsumptionTaxResult("
            f"tax_amount={self.tax_amount}, "
            f"amount_with_tax={self.amount_with_tax}, "
            f"amount_without_tax={self.amount_without_tax}, "
            f"applied_rate={self.applied_rate_numer}/{self.applied_rate_denom}, "
            f"is_reduced_rate={self.is_reduced_rate})"
        )


def calc_consumption_tax(
    amount: int,
    date: datetime.date,
    is_reduced_rate: bool = False,
) -> ConsumptionTaxResult:
    """消費税法第29条に基づく消費税額を計算する。

    # 法的根拠
    消費税法 第29条（税率）

    Args:
        amount (int): 課税標準額（税抜き・円）
        date (datetime.date): 基準日
        is_reduced_rate (bool): 軽減税率フラグ（デフォルト: False）
            2019-10-01以降の飲食料品・新聞等に適用される8%軽減税率。
            WARNING: 対象が軽減税率の適用要件を満たすかの事実認定は呼び出し元の責任。

    Returns:
        ConsumptionTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        r = j_law_uniffi.calc_consumption_tax(
            amount=amount,
            year=date.year,
            month=date.month,
            day=date.day,
            is_reduced_rate=is_reduced_rate,
        )
    except j_law_uniffi.UniError as e:
        raise ValueError(str(e)) from e
    return ConsumptionTaxResult(r)
