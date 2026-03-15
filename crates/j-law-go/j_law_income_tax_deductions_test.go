package jlawcore_test

import (
	"encoding/json"
	"os"
	"testing"
	"time"

	jlawcore "github.com/kmoyashi/j-law-core/crates/j-law-go"
)

type spouseFixture struct {
	SpouseTotalIncomeAmount uint64 `json:"spouse_total_income_amount"`
	IsSameHousehold         bool   `json:"is_same_household"`
	IsElderly               bool   `json:"is_elderly"`
}

type dependentFixture struct {
	GeneralCount           uint64 `json:"general_count"`
	SpecificCount          uint64 `json:"specific_count"`
	ElderlyCohabitingCount uint64 `json:"elderly_cohabiting_count"`
	ElderlyOtherCount      uint64 `json:"elderly_other_count"`
}

type medicalFixture struct {
	MedicalExpensePaid uint64 `json:"medical_expense_paid"`
	ReimbursedAmount   uint64 `json:"reimbursed_amount"`
}

type lifeInsuranceFixture struct {
	NewGeneralPaidAmount           uint64 `json:"new_general_paid_amount"`
	NewIndividualPensionPaidAmount uint64 `json:"new_individual_pension_paid_amount"`
	NewCareMedicalPaidAmount       uint64 `json:"new_care_medical_paid_amount"`
	OldGeneralPaidAmount           uint64 `json:"old_general_paid_amount"`
	OldIndividualPensionPaidAmount uint64 `json:"old_individual_pension_paid_amount"`
}

type donationFixture struct {
	QualifiedDonationAmount uint64 `json:"qualified_donation_amount"`
}

type incomeTaxDeductionFixtureInput struct {
	TotalIncomeAmount          uint64                `json:"total_income_amount"`
	Date                       string                `json:"date"`
	Spouse                     *spouseFixture        `json:"spouse"`
	Dependent                  dependentFixture      `json:"dependent"`
	SocialInsurancePremiumPaid uint64                `json:"social_insurance_premium_paid"`
	Medical                    *medicalFixture       `json:"medical"`
	LifeInsurance              *lifeInsuranceFixture `json:"life_insurance"`
	Donation                   *donationFixture      `json:"donation"`
	ApplyReconstructionTax     bool                  `json:"apply_reconstruction_tax"`
}

type incomeTaxDeductionFixtureExpected struct {
	TotalIncomeAmount             uint64 `json:"total_income_amount"`
	TotalDeductions               uint64 `json:"total_deductions"`
	TaxableIncomeBeforeTruncation uint64 `json:"taxable_income_before_truncation"`
	TaxableIncome                 uint64 `json:"taxable_income"`
	BaseTax                       uint64 `json:"base_tax"`
	ReconstructionTax             uint64 `json:"reconstruction_tax"`
	TotalTax                      uint64 `json:"total_tax"`
}

type incomeTaxDeductionFixtureCase struct {
	ID       string                            `json:"id"`
	Input    incomeTaxDeductionFixtureInput    `json:"input"`
	Expected incomeTaxDeductionFixtureExpected `json:"expected"`
}

type incomeTaxDeductionFixtures struct {
	IncomeTaxDeductions []incomeTaxDeductionFixtureCase `json:"income_tax_deductions"`
	IncomeTaxAssessment []incomeTaxDeductionFixtureCase `json:"income_tax_assessment"`
}

func loadIncomeTaxDeductionFixtures(t *testing.T) incomeTaxDeductionFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/income_tax_deductions.json")
	if err != nil {
		t.Fatalf("failed to read income_tax_deductions.json: %v", err)
	}
	var f incomeTaxDeductionFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse income_tax_deductions.json: %v", err)
	}
	return f
}

func toIncomeDeductionInput(tc incomeTaxDeductionFixtureInput) jlawcore.IncomeDeductionInput {
	parsedDate, _ := time.Parse("2006-01-02", tc.Date)
	input := jlawcore.IncomeDeductionInput{
		TotalIncomeAmount: tc.TotalIncomeAmount,
		Date:              parsedDate,
		Dependent: jlawcore.DependentDeductionInput{
			GeneralCount:           tc.Dependent.GeneralCount,
			SpecificCount:          tc.Dependent.SpecificCount,
			ElderlyCohabitingCount: tc.Dependent.ElderlyCohabitingCount,
			ElderlyOtherCount:      tc.Dependent.ElderlyOtherCount,
		},
		SocialInsurancePremiumPaid: tc.SocialInsurancePremiumPaid,
	}
	if tc.Spouse != nil {
		input.Spouse = &jlawcore.SpouseDeductionInput{
			SpouseTotalIncomeAmount: tc.Spouse.SpouseTotalIncomeAmount,
			IsSameHousehold:         tc.Spouse.IsSameHousehold,
			IsElderly:               tc.Spouse.IsElderly,
		}
	}
	if tc.Medical != nil {
		input.Medical = &jlawcore.MedicalDeductionInput{
			MedicalExpensePaid: tc.Medical.MedicalExpensePaid,
			ReimbursedAmount:   tc.Medical.ReimbursedAmount,
		}
	}
	if tc.LifeInsurance != nil {
		input.LifeInsurance = &jlawcore.LifeInsuranceDeductionInput{
			NewGeneralPaidAmount:           tc.LifeInsurance.NewGeneralPaidAmount,
			NewIndividualPensionPaidAmount: tc.LifeInsurance.NewIndividualPensionPaidAmount,
			NewCareMedicalPaidAmount:       tc.LifeInsurance.NewCareMedicalPaidAmount,
			OldGeneralPaidAmount:           tc.LifeInsurance.OldGeneralPaidAmount,
			OldIndividualPensionPaidAmount: tc.LifeInsurance.OldIndividualPensionPaidAmount,
		}
	}
	if tc.Donation != nil {
		input.Donation = &jlawcore.DonationDeductionInput{
			QualifiedDonationAmount: tc.Donation.QualifiedDonationAmount,
		}
	}
	return input
}

func TestIncomeTaxDeductionsFixtures(t *testing.T) {
	fixtures := loadIncomeTaxDeductionFixtures(t)

	for _, tc := range fixtures.IncomeTaxDeductions {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcIncomeDeductions(toIncomeDeductionInput(tc.Input))
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if result.TotalIncomeAmount != tc.Expected.TotalIncomeAmount {
				t.Errorf("TotalIncomeAmount: got %d, want %d", result.TotalIncomeAmount, tc.Expected.TotalIncomeAmount)
			}
			if result.TotalDeductions != tc.Expected.TotalDeductions {
				t.Errorf("TotalDeductions: got %d, want %d", result.TotalDeductions, tc.Expected.TotalDeductions)
			}
			if result.TaxableIncomeBeforeTruncation != tc.Expected.TaxableIncomeBeforeTruncation {
				t.Errorf("TaxableIncomeBeforeTruncation: got %d, want %d", result.TaxableIncomeBeforeTruncation, tc.Expected.TaxableIncomeBeforeTruncation)
			}
			if result.TaxableIncome != tc.Expected.TaxableIncome {
				t.Errorf("TaxableIncome: got %d, want %d", result.TaxableIncome, tc.Expected.TaxableIncome)
			}
		})
	}
}

func TestIncomeTaxAssessmentFixtures(t *testing.T) {
	fixtures := loadIncomeTaxDeductionFixtures(t)

	for _, tc := range fixtures.IncomeTaxAssessment {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcIncomeTaxAssessment(
				toIncomeDeductionInput(tc.Input),
				tc.Input.ApplyReconstructionTax,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if result.Deductions.TaxableIncome != tc.Expected.TaxableIncome {
				t.Errorf("TaxableIncome: got %d, want %d", result.Deductions.TaxableIncome, tc.Expected.TaxableIncome)
			}
			if result.Tax.BaseTax != tc.Expected.BaseTax {
				t.Errorf("BaseTax: got %d, want %d", result.Tax.BaseTax, tc.Expected.BaseTax)
			}
			if result.Tax.ReconstructionTax != tc.Expected.ReconstructionTax {
				t.Errorf("ReconstructionTax: got %d, want %d", result.Tax.ReconstructionTax, tc.Expected.ReconstructionTax)
			}
			if result.Tax.TotalTax != tc.Expected.TotalTax {
				t.Errorf("TotalTax: got %d, want %d", result.Tax.TotalTax, tc.Expected.TotalTax)
			}
		})
	}
}
