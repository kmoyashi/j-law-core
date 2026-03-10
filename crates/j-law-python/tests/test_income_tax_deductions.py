"""所得控除計算と通し計算のテスト。"""

from __future__ import annotations

import datetime
import json
import pathlib

import pytest

from j_law_python.income_tax import (
    DependentDeductionInput,
    DonationDeductionInput,
    IncomeDeductionInput,
    LifeInsuranceDeductionInput,
    MedicalDeductionInput,
    SpouseDeductionInput,
    calc_income_deductions,
    calc_income_tax_assessment,
)

_FIXTURE_PATH = (
    pathlib.Path(__file__).resolve().parents[3]
    / "tests"
    / "fixtures"
    / "income_tax_deductions.json"
)
_FIXTURES = json.loads(_FIXTURE_PATH.read_text(encoding="utf-8"))


def _build_input(raw: dict) -> IncomeDeductionInput:
    spouse_raw = raw.get("spouse")
    dependent_raw = raw.get("dependent") or {}
    medical_raw = raw.get("medical")
    life_raw = raw.get("life_insurance")
    donation_raw = raw.get("donation")

    return IncomeDeductionInput(
        total_income_amount=raw["total_income_amount"],
        date=datetime.date.fromisoformat(raw["date"]),
        spouse=(
            None
            if spouse_raw is None
            else SpouseDeductionInput(
                spouse_total_income_amount=spouse_raw["spouse_total_income_amount"],
                is_same_household=spouse_raw["is_same_household"],
                is_elderly=spouse_raw["is_elderly"],
            )
        ),
        dependent=DependentDeductionInput(
            general_count=dependent_raw.get("general_count", 0),
            specific_count=dependent_raw.get("specific_count", 0),
            elderly_cohabiting_count=dependent_raw.get("elderly_cohabiting_count", 0),
            elderly_other_count=dependent_raw.get("elderly_other_count", 0),
        ),
        social_insurance_premium_paid=raw.get("social_insurance_premium_paid", 0),
        medical=(
            None
            if medical_raw is None
            else MedicalDeductionInput(
                medical_expense_paid=medical_raw["medical_expense_paid"],
                reimbursed_amount=medical_raw["reimbursed_amount"],
            )
        ),
        life_insurance=(
            None
            if life_raw is None
            else LifeInsuranceDeductionInput(
                new_general_paid_amount=life_raw["new_general_paid_amount"],
                new_individual_pension_paid_amount=life_raw[
                    "new_individual_pension_paid_amount"
                ],
                new_care_medical_paid_amount=life_raw["new_care_medical_paid_amount"],
                old_general_paid_amount=life_raw["old_general_paid_amount"],
                old_individual_pension_paid_amount=life_raw[
                    "old_individual_pension_paid_amount"
                ],
            )
        ),
        donation=(
            None
            if donation_raw is None
            else DonationDeductionInput(
                qualified_donation_amount=donation_raw["qualified_donation_amount"]
            )
        ),
    )


@pytest.mark.parametrize(
    "case",
    _FIXTURES["income_tax_deductions"],
    ids=lambda c: c["id"],
)
def test_income_tax_deductions(case):
    result = calc_income_deductions(_build_input(case["input"]))
    expected = case["expected"]

    assert result.total_income_amount == expected["total_income_amount"]
    assert result.total_deductions == expected["total_deductions"]
    assert (
        result.taxable_income_before_truncation
        == expected["taxable_income_before_truncation"]
    )
    assert result.taxable_income == expected["taxable_income"]


@pytest.mark.parametrize(
    "case",
    _FIXTURES["income_tax_assessment"],
    ids=lambda c: c["id"],
)
def test_income_tax_assessment(case):
    result = calc_income_tax_assessment(
        _build_input(case["input"]),
        apply_reconstruction_tax=case["input"]["apply_reconstruction_tax"],
    )
    expected = case["expected"]

    assert result.deductions.taxable_income == expected["taxable_income"]
    assert result.tax.base_tax == expected["base_tax"]
    assert result.tax.reconstruction_tax == expected["reconstruction_tax"]
    assert result.tax.total_tax == expected["total_tax"]


def test_income_tax_deductions_reject_invalid_date_type():
    with pytest.raises(TypeError):
        calc_income_deductions(  # type: ignore[arg-type]
            IncomeDeductionInput(total_income_amount=1, date="2024-01-01")
        )
