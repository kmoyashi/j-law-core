"""所得税法第204条第1項に基づく源泉徴収税額計算のテスト。"""

from __future__ import annotations

import datetime
import json
import pathlib

import pytest

from j_law_python.withholding_tax import (
    WithholdingTaxCategory,
    calc_withholding_tax,
)

_FIXTURE_PATH = (
    pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "withholding_tax.json"
)
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


@pytest.mark.parametrize("case", _FIXTURES["withholding_tax"], ids=lambda c: c["id"])
def test_withholding_tax(case):
    inp = case["input"]
    exp = case["expected"]
    date = datetime.date.fromisoformat(inp["date"])

    result = calc_withholding_tax(
        inp["payment_amount"],
        date,
        inp["category"],
        is_submission_prize=inp["is_submission_prize"],
        separated_consumption_tax_amount=inp["separated_consumption_tax_amount"],
    )

    assert result.taxable_payment_amount == exp["taxable_payment_amount"]
    assert result.tax_amount == exp["tax_amount"]
    assert result.net_payment_amount == exp["net_payment_amount"]
    assert result.submission_prize_exempted is exp["submission_prize_exempted"]


class TestLanguageSpecific:
    def test_out_of_range_date(self):
        with pytest.raises(ValueError):
            calc_withholding_tax(
                100_000,
                datetime.date(2012, 12, 31),
                WithholdingTaxCategory.MANUSCRIPT_AND_LECTURE,
            )

    def test_invalid_date_type(self):
        with pytest.raises(TypeError):
            calc_withholding_tax(100_000, "2026-01-01", "professional_fee")

    def test_breakdown_fields(self):
        result = calc_withholding_tax(
            1_500_000,
            datetime.date(2026, 1, 1),
            "professional_fee",
        )
        assert len(result.breakdown) == 2
        assert result.breakdown[0]["label"] != ""

    def test_repr(self):
        result = calc_withholding_tax(
            100_000,
            datetime.date(2026, 1, 1),
            "manuscript_and_lecture",
        )
        assert "WithholdingTaxResult" in repr(result)
