"""印紙税法別表第一に基づく印紙税額計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime

import j_law_uniffi


def calc_stamp_tax(
    contract_amount: int,
    date: datetime.date,
    is_reduced_rate_applicable: bool = False,
) -> j_law_uniffi.StampTaxResult:
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
        j_law_uniffi.StampTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        return j_law_uniffi.calc_stamp_tax(
            contract_amount=contract_amount,
            year=date.year,
            month=date.month,
            day=date.day,
            is_reduced_rate_applicable=is_reduced_rate_applicable,
        )
    except j_law_uniffi.JLawError as e:
        raise ValueError(str(e)) from e
