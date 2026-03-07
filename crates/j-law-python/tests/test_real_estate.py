"""宅建業法第46条に基づく媒介報酬計算のテスト。

法的根拠: 宅地建物取引業法 第46条第1項 / 国土交通省告示

テストケースは tests/fixtures/real_estate.json から読み込む。
"""

import datetime
import json
import pathlib

import pytest

from j_law_python.real_estate import calc_brokerage_fee

# ─── フィクスチャ読み込み ─────────────────────────────────────────────────────

_FIXTURE_PATH = pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "real_estate.json"
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


# ─── データ駆動テスト ─────────────────────────────────────────────────────────


@pytest.mark.parametrize("case", _FIXTURES["brokerage_fee"], ids=lambda c: c["id"])
def test_brokerage_fee(case):
    inp = case["input"]
    exp = case["expected"]

    date = datetime.date.fromisoformat(inp["date"])
    r = calc_brokerage_fee(
        inp["price"],
        date,
        is_low_cost_vacant_house=inp["is_low_cost_vacant_house"],
        is_seller=inp.get("is_seller", False),
    )

    if "total_without_tax" in exp:
        assert r.total_without_tax == exp["total_without_tax"], f"{case['id']}: total_without_tax"
    if "tax_amount" in exp:
        assert r.tax_amount == exp["tax_amount"], f"{case['id']}: tax_amount"
    if "total_with_tax" in exp:
        assert r.total_with_tax == exp["total_with_tax"], f"{case['id']}: total_with_tax"
    if "low_cost_special_applied" in exp:
        assert r.low_cost_special_applied is exp["low_cost_special_applied"], f"{case['id']}: low_cost_special_applied"
    if "breakdown_results" in exp:
        actual = [step.result for step in r.breakdown]
        assert actual == exp["breakdown_results"], f"{case['id']}: breakdown_results"


# ─── 言語固有テスト（JSON の外） ──────────────────────────────────────────────


class TestLanguageSpecific:
    """Python 固有の振る舞い検証。"""

    def test_error_date_out_of_range(self):
        # 1970-12-01 施行のため、それ以前の日付はエラー
        with pytest.raises(ValueError, match="1970-11-30"):
            calc_brokerage_fee(5_000_000, datetime.date(1970, 11, 30))

    def test_breakdown_fields(self):
        r = calc_brokerage_fee(5_000_000, datetime.date(2024, 8, 1))
        for step in r.breakdown:
            assert step.label != ""
            assert step.rate_denom > 0

    def test_repr(self):
        r = calc_brokerage_fee(5_000_000, datetime.date(2024, 8, 1))
        assert "BrokerageFeeResult" in repr(r)

    def test_type_error_invalid_date(self):
        """date に datetime.date 以外を渡すと TypeError。"""
        with pytest.raises(TypeError):
            calc_brokerage_fee(5_000_000, "2024-08-01")
        with pytest.raises(TypeError):
            calc_brokerage_fee(5_000_000, 20240801)
