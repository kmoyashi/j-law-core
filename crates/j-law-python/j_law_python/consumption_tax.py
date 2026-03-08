"""消費税法第29条に基づく消費税額計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime

import j_law_uniffi


def calc_consumption_tax(
    amount: int,
    date: datetime.date,
    is_reduced_rate: bool = False,
) -> j_law_uniffi.ConsumptionTaxResult:
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
        j_law_uniffi.ConsumptionTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 軽減税率フラグが指定されたが対象日に軽減税率が存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        return j_law_uniffi.calc_consumption_tax(
            amount=amount,
            year=date.year,
            month=date.month,
            day=date.day,
            is_reduced_rate=is_reduced_rate,
        )
    except j_law_uniffi.JLawError as e:
        raise ValueError(str(e)) from e
