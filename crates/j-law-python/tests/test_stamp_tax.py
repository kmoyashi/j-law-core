"""印紙税法 別表第一に基づく印紙税額計算のテスト。

法的根拠: 印紙税法 別表第一 第1号文書 / 租税特別措置法 第91条

テストケースは tests/fixtures/stamp_tax.json から読み込む。
"""

import datetime
import json
import pathlib

import pytest

from j_law_python.stamp_tax import calc_stamp_tax

# ─── フィクスチャ読み込み ─────────────────────────────────────────────────────

_FIXTURE_PATH = pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "stamp_tax.json"
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


# ─── データ駆動テスト ─────────────────────────────────────────────────────────


@pytest.mark.parametrize("case", _FIXTURES["stamp_tax"], ids=lambda c: c["id"])
def test_stamp_tax(case):
    inp = case["input"]
    exp = case["expected"]

    date = datetime.date.fromisoformat(inp["date"])
    r = calc_stamp_tax(
        inp["contract_amount"],
        date,
        is_reduced_rate_applicable=inp["is_reduced_rate_applicable"],
    )

    if "tax_amount" in exp:
        assert r.tax_amount == exp["tax_amount"], f"{case['id']}: tax_amount"
    if "reduced_rate_applied" in exp:
        assert r.reduced_rate_applied is exp["reduced_rate_applied"], f"{case['id']}: reduced_rate_applied"


# ─── 言語固有テスト（JSON の外） ──────────────────────────────────────────────


class TestLanguageSpecific:
    """Python 固有の振る舞い検証。"""

    def test_error_date_out_of_range(self):
        with pytest.raises(ValueError, match="2014-03-31"):
            calc_stamp_tax(5_000_000, datetime.date(2014, 3, 31))

    def test_repr(self):
        r = calc_stamp_tax(5_000_000, datetime.date(2024, 8, 1))
        assert "StampTaxResult" in type(r).__name__

    def test_type_error_invalid_date(self):
        """date に datetime.date 以外を渡すと TypeError。"""
        with pytest.raises(TypeError):
            calc_stamp_tax(5_000_000, "2024-08-01")
        with pytest.raises(TypeError):
            calc_stamp_tax(5_000_000, 20240801)
