"""所得税法第89条に基づく所得税計算（Python ラッパー）。

UniFFI バインディング（j_law_uniffi）をラップし、
datetime.date を受け取るインターフェースを提供する。
"""

from __future__ import annotations

import datetime

import j_law_uniffi


def calc_income_tax(
    taxable_income: int,
    date: datetime.date,
    apply_reconstruction_tax: bool = True,
) -> j_law_uniffi.IncomeTaxResult:
    """所得税法第89条に基づく所得税額を計算する。

    # 法的根拠
    所得税法 第89条第1項 / 復興財源確保法 第13条

    Args:
        taxable_income (int): 課税所得金額（円・1,000円未満切り捨て済み）
        date (datetime.date): 基準日
        apply_reconstruction_tax (bool): 復興特別所得税を適用するか（デフォルト: True）

    Returns:
        j_law_uniffi.IncomeTaxResult

    Raises:
        TypeError: date が datetime.date 型でない場合
        ValueError: 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合
    """
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )
    try:
        return j_law_uniffi.calc_income_tax(
            taxable_income=taxable_income,
            year=date.year,
            month=date.month,
            day=date.day,
            apply_reconstruction_tax=apply_reconstruction_tax,
        )
    except j_law_uniffi.JLawError as e:
        raise ValueError(str(e)) from e
