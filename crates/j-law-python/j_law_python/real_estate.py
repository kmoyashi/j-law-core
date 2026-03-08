"""宅地建物取引業法第46条に基づく媒介報酬計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime

import j_law_uniffi


def calc_brokerage_fee(
    price: int,
    date: datetime.date,
    is_low_cost_vacant_house: bool = False,
    is_seller: bool = False,
) -> j_law_uniffi.BrokerageFeeResult:
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
        j_law_uniffi.BrokerageFeeResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        return j_law_uniffi.calc_brokerage_fee(
            price=price,
            year=date.year,
            month=date.month,
            day=date.day,
            is_low_cost_vacant_house=is_low_cost_vacant_house,
            is_seller=is_seller,
        )
    except j_law_uniffi.JLawError as e:
        raise ValueError(str(e)) from e
