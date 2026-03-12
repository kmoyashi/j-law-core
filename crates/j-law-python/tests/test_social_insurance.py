"""社会保険料計算のテスト。"""

import datetime
import json
import pathlib

import pytest

from j_law_python.social_insurance import calc_social_insurance

_FIXTURE_PATH = (
    pathlib.Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "social_insurance.json"
)
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


@pytest.mark.parametrize("case", _FIXTURES["social_insurance"], ids=lambda c: c["id"])
def test_social_insurance(case):
    inp = case["input"]
    exp = case["expected"]

    result = calc_social_insurance(
        inp["standard_monthly_remuneration"],
        datetime.date.fromisoformat(inp["date"]),
        inp["prefecture_code"],
        is_care_insurance_applicable=inp["is_care_insurance_applicable"],
    )

    assert result.health_related_amount == exp["health_related_amount"]
    assert result.pension_amount == exp["pension_amount"]
    assert result.total_amount == exp["total_amount"]
    assert result.care_insurance_applied is exp["care_insurance_applied"]


def test_social_insurance_invalid_standard_monthly_remuneration():
    with pytest.raises(ValueError, match="標準報酬月額"):
        calc_social_insurance(305_000, datetime.date(2026, 3, 1), 13)


def test_social_insurance_type_error_invalid_date():
    with pytest.raises(TypeError):
        calc_social_insurance(118_000, "2026-03-01", 13)
