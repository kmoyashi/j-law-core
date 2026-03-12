package jlawcore_test

import (
	"encoding/json"
	"os"
	"testing"

	jlawcore "github.com/kmoyashi/j-law-go"
)

type socialInsuranceInput struct {
	StandardMonthlyRemuneration uint64 `json:"standard_monthly_remuneration"`
	Date                        string `json:"date"`
	PrefectureCode              uint8  `json:"prefecture_code"`
	IsCareInsuranceApplicable   bool   `json:"is_care_insurance_applicable"`
}

type socialInsuranceExpected struct {
	HealthRelatedAmount  uint64 `json:"health_related_amount"`
	PensionAmount        uint64 `json:"pension_amount"`
	TotalAmount          uint64 `json:"total_amount"`
	CareInsuranceApplied bool   `json:"care_insurance_applied"`
}

type socialInsuranceCase struct {
	ID          string                  `json:"id"`
	Description string                  `json:"description"`
	Input       socialInsuranceInput    `json:"input"`
	Expected    socialInsuranceExpected `json:"expected"`
}

type socialInsuranceFixtures struct {
	SocialInsurance []socialInsuranceCase `json:"social_insurance"`
}

func loadSocialInsuranceFixtures(t *testing.T) socialInsuranceFixtures {
	t.Helper()
	data, err := os.ReadFile("../../tests/fixtures/social_insurance.json")
	if err != nil {
		t.Fatalf("failed to read social_insurance.json: %v", err)
	}
	var f socialInsuranceFixtures
	if err := json.Unmarshal(data, &f); err != nil {
		t.Fatalf("failed to parse social_insurance.json: %v", err)
	}
	return f
}

func TestSocialInsurance(t *testing.T) {
	fixtures := loadSocialInsuranceFixtures(t)

	for _, tc := range fixtures.SocialInsurance {
		t.Run(tc.ID, func(t *testing.T) {
			result, err := jlawcore.CalcSocialInsurance(
				tc.Input.StandardMonthlyRemuneration,
				parseDate(t, tc.Input.Date),
				tc.Input.PrefectureCode,
				tc.Input.IsCareInsuranceApplicable,
			)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if result.HealthRelatedAmount != tc.Expected.HealthRelatedAmount {
				t.Fatalf("HealthRelatedAmount: got %d want %d", result.HealthRelatedAmount, tc.Expected.HealthRelatedAmount)
			}
			if result.PensionAmount != tc.Expected.PensionAmount {
				t.Fatalf("PensionAmount: got %d want %d", result.PensionAmount, tc.Expected.PensionAmount)
			}
			if result.TotalAmount != tc.Expected.TotalAmount {
				t.Fatalf("TotalAmount: got %d want %d", result.TotalAmount, tc.Expected.TotalAmount)
			}
			if result.CareInsuranceApplied != tc.Expected.CareInsuranceApplied {
				t.Fatalf("CareInsuranceApplied: got %v want %v", result.CareInsuranceApplied, tc.Expected.CareInsuranceApplied)
			}
		})
	}
}

