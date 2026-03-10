"""所得税計算と所得控除計算の Python ラッパー。"""

from __future__ import annotations

import datetime
from dataclasses import dataclass, field
from typing import List

from ._c_ffi import CFFIError
from ._c_ffi import (
    IncomeDeductionInputRecord,
    IncomeDeductionLineRecord,
    IncomeDeductionRecord,
    IncomeTaxAssessmentRecord,
    IncomeTaxRecord,
    IncomeTaxStepRecord,
)
from ._c_ffi import calc_income_deductions as _calc_income_deductions
from ._c_ffi import calc_income_tax as _calc_income_tax
from ._c_ffi import calc_income_tax_assessment as _calc_income_tax_assessment


@dataclass(frozen=True)
class SpouseDeductionInput:
    spouse_total_income_amount: int
    is_same_household: bool
    is_elderly: bool = False


@dataclass(frozen=True)
class DependentDeductionInput:
    general_count: int = 0
    specific_count: int = 0
    elderly_cohabiting_count: int = 0
    elderly_other_count: int = 0


@dataclass(frozen=True)
class MedicalDeductionInput:
    medical_expense_paid: int
    reimbursed_amount: int = 0


@dataclass(frozen=True)
class LifeInsuranceDeductionInput:
    new_general_paid_amount: int = 0
    new_individual_pension_paid_amount: int = 0
    new_care_medical_paid_amount: int = 0
    old_general_paid_amount: int = 0
    old_individual_pension_paid_amount: int = 0


@dataclass(frozen=True)
class DonationDeductionInput:
    qualified_donation_amount: int


@dataclass(frozen=True)
class IncomeDeductionInput:
    total_income_amount: int
    date: datetime.date
    spouse: SpouseDeductionInput | None = None
    dependent: DependentDeductionInput = field(default_factory=DependentDeductionInput)
    social_insurance_premium_paid: int = 0
    medical: MedicalDeductionInput | None = None
    life_insurance: LifeInsuranceDeductionInput | None = None
    donation: DonationDeductionInput | None = None


class IncomeTaxStep:
    """所得税速算表の1ブラケット分の内訳。"""

    def __init__(self, r: IncomeTaxStepRecord) -> None:
        self.label: str = r.label
        self.taxable_income: int = r.taxable_income
        self.rate_numer: int = r.rate_numer
        self.rate_denom: int = r.rate_denom
        self.deduction: int = r.deduction
        self.result: int = r.result

    def __repr__(self) -> str:
        return (
            f"IncomeTaxStep("
            f"label={self.label!r}, "
            f"taxable_income={self.taxable_income}, "
            f"rate={self.rate_numer}/{self.rate_denom}, "
            f"deduction={self.deduction}, "
            f"result={self.result})"
        )


class IncomeTaxResult:
    """所得税の計算結果。"""

    def __init__(self, r: IncomeTaxRecord) -> None:
        self.base_tax: int = r.base_tax
        self.reconstruction_tax: int = r.reconstruction_tax
        self.total_tax: int = r.total_tax
        self.reconstruction_tax_applied: bool = r.reconstruction_tax_applied
        self.breakdown: List[IncomeTaxStep] = [IncomeTaxStep(s) for s in r.breakdown]

    def __repr__(self) -> str:
        return (
            f"IncomeTaxResult("
            f"base_tax={self.base_tax}, "
            f"reconstruction_tax={self.reconstruction_tax}, "
            f"total_tax={self.total_tax}, "
            f"reconstruction_tax_applied={self.reconstruction_tax_applied})"
        )


class IncomeDeductionLine:
    """所得控除の内訳1行。"""

    def __init__(self, r: IncomeDeductionLineRecord) -> None:
        self.kind: int = r.kind
        self.label: str = r.label
        self.amount: int = r.amount

    def __repr__(self) -> str:
        return (
            f"IncomeDeductionLine(kind={self.kind}, "
            f"label={self.label!r}, amount={self.amount})"
        )


class IncomeDeductionResult:
    """所得控除の計算結果。"""

    def __init__(self, r: IncomeDeductionRecord) -> None:
        self.total_income_amount: int = r.total_income_amount
        self.total_deductions: int = r.total_deductions
        self.taxable_income_before_truncation: int = (
            r.taxable_income_before_truncation
        )
        self.taxable_income: int = r.taxable_income
        self.breakdown: List[IncomeDeductionLine] = [
            IncomeDeductionLine(s) for s in r.breakdown
        ]

    def __repr__(self) -> str:
        return (
            f"IncomeDeductionResult("
            f"total_income_amount={self.total_income_amount}, "
            f"total_deductions={self.total_deductions}, "
            f"taxable_income={self.taxable_income})"
        )


class IncomeTaxAssessmentResult:
    """所得控除から所得税額までの通し計算結果。"""

    def __init__(self, r: IncomeTaxAssessmentRecord) -> None:
        self.deductions = IncomeDeductionResult(r.deductions)
        self.tax = IncomeTaxResult(r.tax)

    def __repr__(self) -> str:
        return (
            "IncomeTaxAssessmentResult("
            f"taxable_income={self.deductions.taxable_income}, "
            f"total_tax={self.tax.total_tax})"
        )


def _ensure_date(date: datetime.date) -> None:
    if not isinstance(date, datetime.date):
        raise TypeError(
            f"date には datetime.date を指定してください (got {type(date).__name__})"
        )


def _to_record(input_data: IncomeDeductionInput) -> IncomeDeductionInputRecord:
    _ensure_date(input_data.date)

    spouse = input_data.spouse
    medical = input_data.medical
    life_insurance = input_data.life_insurance
    donation = input_data.donation
    dependent = input_data.dependent

    return IncomeDeductionInputRecord(
        total_income_amount=input_data.total_income_amount,
        year=input_data.date.year,
        month=input_data.date.month,
        day=input_data.date.day,
        has_spouse=spouse is not None,
        spouse_total_income_amount=0 if spouse is None else spouse.spouse_total_income_amount,
        spouse_is_same_household=False if spouse is None else spouse.is_same_household,
        spouse_is_elderly=False if spouse is None else spouse.is_elderly,
        dependent_general_count=dependent.general_count,
        dependent_specific_count=dependent.specific_count,
        dependent_elderly_cohabiting_count=dependent.elderly_cohabiting_count,
        dependent_elderly_other_count=dependent.elderly_other_count,
        social_insurance_premium_paid=input_data.social_insurance_premium_paid,
        has_medical=medical is not None,
        medical_expense_paid=0 if medical is None else medical.medical_expense_paid,
        medical_reimbursed_amount=0 if medical is None else medical.reimbursed_amount,
        has_life_insurance=life_insurance is not None,
        life_new_general_paid_amount=(
            0 if life_insurance is None else life_insurance.new_general_paid_amount
        ),
        life_new_individual_pension_paid_amount=(
            0
            if life_insurance is None
            else life_insurance.new_individual_pension_paid_amount
        ),
        life_new_care_medical_paid_amount=(
            0 if life_insurance is None else life_insurance.new_care_medical_paid_amount
        ),
        life_old_general_paid_amount=(
            0 if life_insurance is None else life_insurance.old_general_paid_amount
        ),
        life_old_individual_pension_paid_amount=(
            0
            if life_insurance is None
            else life_insurance.old_individual_pension_paid_amount
        ),
        has_donation=donation is not None,
        donation_qualified_amount=(
            0 if donation is None else donation.qualified_donation_amount
        ),
    )


def calc_income_tax(
    taxable_income: int,
    date: datetime.date,
    apply_reconstruction_tax: bool = True,
) -> IncomeTaxResult:
    """課税所得金額から所得税額を計算する。"""
    _ensure_date(date)
    try:
        r = _calc_income_tax(
            taxable_income,
            date.year,
            date.month,
            date.day,
            apply_reconstruction_tax,
        )
    except CFFIError as e:
        raise ValueError(str(e)) from e
    return IncomeTaxResult(r)


def calc_income_deductions(input_data: IncomeDeductionInput) -> IncomeDeductionResult:
    """総所得金額等から所得控除を計算し、課税所得金額を返す。"""
    try:
        r = _calc_income_deductions(_to_record(input_data))
    except CFFIError as e:
        raise ValueError(str(e)) from e
    return IncomeDeductionResult(r)


def calc_income_tax_assessment(
    input_data: IncomeDeductionInput,
    apply_reconstruction_tax: bool = True,
) -> IncomeTaxAssessmentResult:
    """所得控除から所得税額までを通しで計算する。"""
    try:
        r = _calc_income_tax_assessment(
            _to_record(input_data),
            apply_reconstruction_tax,
        )
    except CFFIError as e:
        raise ValueError(str(e)) from e
    return IncomeTaxAssessmentResult(r)
