"""宅地建物取引業法第46条に基づく媒介報酬計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime
from typing import List

import j_law_uniffi


class BreakdownStep:
    """1ティアの計算内訳。

    Attributes:
        label (str): ティアの表示名
        base_amount (int): ティア対象金額（円）
        rate_numer (int): 適用レートの分子
        rate_denom (int): 適用レートの分母
        result (int): ティア計算結果（円・端数切捨て済み）
    """

    def __init__(self, r: j_law_uniffi.UniBreakdownStep) -> None:
        self.label: str = r.label
        self.base_amount: int = r.base_amount
        self.rate_numer: int = r.rate_numer
        self.rate_denom: int = r.rate_denom
        self.result: int = r.result

    def __repr__(self) -> str:
        return (
            f"BreakdownStep("
            f"label={self.label!r}, "
            f"base_amount={self.base_amount}, "
            f"rate={self.rate_numer}/{self.rate_denom}, "
            f"result={self.result})"
        )


class BrokerageFeeResult:
    """媒介報酬の計算結果。

    Attributes:
        total_without_tax (int): 税抜合計額（円）
        total_with_tax (int): 税込合計額（円）
        tax_amount (int): 消費税額（円）
        low_cost_special_applied (bool): 低廉な空き家特例が適用されたか
        breakdown (list[BreakdownStep]): 各ティアの計算内訳
    """

    def __init__(self, r: j_law_uniffi.UniBrokerageFeeResult) -> None:
        self.total_without_tax: int = r.total_without_tax
        self.total_with_tax: int = r.total_with_tax
        self.tax_amount: int = r.tax_amount
        self.low_cost_special_applied: bool = r.low_cost_special_applied
        self.breakdown: List[BreakdownStep] = [BreakdownStep(s) for s in r.breakdown]

    def __repr__(self) -> str:
        return (
            f"BrokerageFeeResult("
            f"total_without_tax={self.total_without_tax}, "
            f"total_with_tax={self.total_with_tax}, "
            f"tax_amount={self.tax_amount}, "
            f"low_cost_special_applied={self.low_cost_special_applied})"
        )


def calc_brokerage_fee(
    price: int,
    date: datetime.date,
    is_low_cost_vacant_house: bool = False,
    is_seller: bool = False,
) -> BrokerageFeeResult:
    """宅建業法第46条に基づく媒介報酬を計算する。

    # 法的根拠
    宅地建物取引業法 第46条第1項 / 国土交通省告示

    Args:
        price (int): 売買価格（円）
        date (datetime.date): 基準日
        is_low_cost_vacant_house (bool): 低廉な空き家特例フラグ（デフォルト: False）
            WARNING: 対象物件が「低廉な空き家」に該当するかの事実認定は呼び出し元の責任。
        is_seller (bool): 売主側フラグ（デフォルト: False）
            2018年1月1日〜2024年6月30日の低廉特例は売主のみに適用される。
            WARNING: 売主・買主の事実認定は呼び出し元の責任。

    Returns:
        BrokerageFeeResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        r = j_law_uniffi.calc_brokerage_fee(
            price=price,
            year=date.year,
            month=date.month,
            day=date.day,
            is_low_cost_vacant_house=is_low_cost_vacant_house,
            is_seller=is_seller,
        )
    except j_law_uniffi.UniError as e:
        raise ValueError(str(e)) from e
    return BrokerageFeeResult(r)
