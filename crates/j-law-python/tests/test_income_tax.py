"""所得税法第89条に基づく所得税計算のテスト。

法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条

テストケースは tests/fixtures/income_tax.json から読み込む。
"""

import json
import pathlib

import pytest

from j_law_python.income_tax import calc_income_tax

# ─── フィクスチャ読み込み ─────────────────────────────────────────────────────

_FIXTURE_PATH = pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "income_tax.json"
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


# ─── データ駆動テスト ─────────────────────────────────────────────────────────


@pytest.mark.parametrize("case", _FIXTURES["income_tax"], ids=lambda c: c["id"])
def test_income_tax(case):
    inp = case["input"]
    exp = case["expected"]

    r = calc_income_tax(
        inp["taxable_income"],
        inp["year"],
        inp["month"],
        inp["day"],
        apply_reconstruction_tax=inp["apply_reconstruction_tax"],
    )

    assert r.base_tax == exp["base_tax"], f"{case['id']}: base_tax"
    assert r.reconstruction_tax == exp["reconstruction_tax"], f"{case['id']}: reconstruction_tax"
    assert r.total_tax == exp["total_tax"], f"{case['id']}: total_tax"
    assert r.reconstruction_tax_applied is exp["reconstruction_tax_applied"], f"{case['id']}: reconstruction_tax_applied"


# ─── 言語固有テスト（JSON の外） ──────────────────────────────────────────────


class TestLanguageSpecific:
    """Python 固有の振る舞い検証。"""

    def test_error_date_out_of_range(self):
        with pytest.raises(ValueError):
            calc_income_tax(5_000_000, 2014, 12, 31)

    def test_breakdown_fields(self):
        r = calc_income_tax(5_000_000, 2024, 1, 1)
        assert len(r.breakdown) > 0
        for step in r.breakdown:
            assert step.label != ""
            assert step.rate_denom > 0

    def test_repr(self):
        r = calc_income_tax(5_000_000, 2024, 1, 1)
        assert "IncomeTaxResult" in repr(r)
