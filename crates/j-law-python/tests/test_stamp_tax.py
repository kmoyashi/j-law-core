"""印紙税法 別表第一に基づく印紙税額計算のテスト。"""

import datetime
import json
import pathlib

import pytest

from j_law_python.stamp_tax import calc_stamp_tax

_FIXTURE_PATH = pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "stamp_tax.json"
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


@pytest.mark.parametrize("case", _FIXTURES["stamp_tax"], ids=lambda c: c["id"])
def test_stamp_tax(case):
    inp = case["input"]
    exp = case["expected"]

    date = datetime.date.fromisoformat(inp["date"])
    r = calc_stamp_tax(
        inp["document_code"],
        inp["stated_amount"],
        date,
        flags=inp["flags"],
    )

    assert r.tax_amount == exp["tax_amount"], f"{case['id']}: tax_amount"
    assert r.rule_label == exp["rule_label"], f"{case['id']}: rule_label"
    assert r.applied_special_rule == exp["applied_special_rule"], f"{case['id']}: applied_special_rule"


class TestLanguageSpecific:
    def test_error_date_out_of_range(self):
        with pytest.raises(ValueError, match="2014-03-31"):
            calc_stamp_tax(
                "article1_real_estate_transfer",
                5_000_000,
                datetime.date(2014, 3, 31),
            )

    def test_repr(self):
        r = calc_stamp_tax(
            "article1_real_estate_transfer",
            5_000_000,
            datetime.date(2024, 8, 1),
        )
        assert "StampTaxResult" in repr(r)

    def test_type_error_invalid_date(self):
        with pytest.raises(TypeError):
            calc_stamp_tax("article1_real_estate_transfer", 5_000_000, "2024-08-01")

    def test_invalid_document_code(self):
        with pytest.raises(ValueError, match="document_code"):
            calc_stamp_tax("invalid_code", 5_000_000, datetime.date(2024, 8, 1))

    def test_invalid_flag(self):
        with pytest.raises(ValueError, match="flag"):
            calc_stamp_tax(
                "article17_sales_receipt",
                70_000,
                datetime.date(2024, 8, 1),
                flags=["invalid_flag"],
            )
