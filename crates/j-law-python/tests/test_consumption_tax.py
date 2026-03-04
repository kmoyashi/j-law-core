"""消費税法第29条に基づく消費税額計算のテスト。

法的根拠: 消費税法 第29条（税率）

テストケースは tests/fixtures/consumption_tax.json から読み込む。
"""

import json
import pathlib

import pytest

from j_law_python.consumption_tax import calc_consumption_tax

# ─── フィクスチャ読み込み ─────────────────────────────────────────────────────

_FIXTURE_PATH = pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "consumption_tax.json"
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


# ─── データ駆動テスト ─────────────────────────────────────────────────────────


@pytest.mark.parametrize("case", _FIXTURES["consumption_tax"], ids=lambda c: c["id"])
def test_consumption_tax(case):
    inp = case["input"]
    exp = case["expected"]

    r = calc_consumption_tax(
        inp["amount"],
        inp["year"],
        inp["month"],
        inp["day"],
        is_reduced_rate=inp["is_reduced_rate"],
    )

    if "tax_amount" in exp:
        assert r.tax_amount == exp["tax_amount"], f"{case['id']}: tax_amount"
    if "amount_with_tax" in exp:
        assert r.amount_with_tax == exp["amount_with_tax"], f"{case['id']}: amount_with_tax"
    if "amount_without_tax" in exp:
        assert r.amount_without_tax == exp["amount_without_tax"], f"{case['id']}: amount_without_tax"
    if "applied_rate_numer" in exp:
        assert r.applied_rate_numer == exp["applied_rate_numer"], f"{case['id']}: applied_rate_numer"
    if "applied_rate_denom" in exp:
        assert r.applied_rate_denom == exp["applied_rate_denom"], f"{case['id']}: applied_rate_denom"
    if "is_reduced_rate" in exp:
        assert r.is_reduced_rate is exp["is_reduced_rate"], f"{case['id']}: is_reduced_rate"


# ─── 言語固有テスト（JSON の外） ──────────────────────────────────────────────


class TestLanguageSpecific:
    """Python 固有の振る舞い検証。"""

    def test_error_reduced_rate_without_support(self):
        """軽減税率フラグを立てても対応期間外ならエラー。"""
        with pytest.raises(ValueError):
            calc_consumption_tax(100_000, 2016, 1, 1, is_reduced_rate=True)

    def test_repr(self):
        r = calc_consumption_tax(100_000, 2024, 1, 1)
        assert "ConsumptionTaxResult" in repr(r)

    def test_before_introduction_no_tax(self):
        """消費税導入前は税額ゼロで正常終了（エラーにならない）。"""
        r = calc_consumption_tax(100_000, 1988, 1, 1)
        assert r.tax_amount == 0
        assert r.amount_with_tax == 100_000
