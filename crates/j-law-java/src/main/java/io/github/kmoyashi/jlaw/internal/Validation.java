package io.github.kmoyashi.jlaw.internal;

import io.github.kmoyashi.jlaw.DependentDeductionInput;
import io.github.kmoyashi.jlaw.DonationDeductionInput;
import io.github.kmoyashi.jlaw.IncomeDeductionInput;
import io.github.kmoyashi.jlaw.LifeInsuranceDeductionInput;
import io.github.kmoyashi.jlaw.MedicalDeductionInput;
import io.github.kmoyashi.jlaw.SpouseDeductionInput;
import java.time.LocalDate;
import java.util.Objects;

public final class Validation {
    private static final int MAX_U16 = 65535;

    private Validation() {
    }

    public static LocalDate requireDate(LocalDate date, String fieldName) {
        Objects.requireNonNull(date, fieldName + " must not be null");
        int year = date.getYear();
        if (year < 0 || year > MAX_U16) {
            throw new IllegalArgumentException(fieldName + ".year must be between 0 and 65535");
        }
        return date;
    }

    public static long requireNonNegative(long value, String fieldName) {
        if (value < 0L) {
            throw new IllegalArgumentException(fieldName + " must be non-negative");
        }
        return value;
    }

    public static void validateIncomeDeductionInput(IncomeDeductionInput input) {
        Objects.requireNonNull(input, "input must not be null");
        requireNonNegative(input.getTotalIncomeAmount(), "input.totalIncomeAmount");
        requireDate(input.getDate(), "input.date");
        requireNonNegative(input.getSocialInsurancePremiumPaid(), "input.socialInsurancePremiumPaid");

        SpouseDeductionInput spouse = input.getSpouse();
        if (spouse != null) {
            requireNonNegative(spouse.getSpouseTotalIncomeAmount(), "input.spouse.spouseTotalIncomeAmount");
        }

        DependentDeductionInput dependent = input.getDependent();
        Objects.requireNonNull(dependent, "input.dependent must not be null");
        requireNonNegative(dependent.getGeneralCount(), "input.dependent.generalCount");
        requireNonNegative(dependent.getSpecificCount(), "input.dependent.specificCount");
        requireNonNegative(dependent.getElderlyCohabitingCount(), "input.dependent.elderlyCohabitingCount");
        requireNonNegative(dependent.getElderlyOtherCount(), "input.dependent.elderlyOtherCount");

        MedicalDeductionInput medical = input.getMedical();
        if (medical != null) {
            requireNonNegative(medical.getMedicalExpensePaid(), "input.medical.medicalExpensePaid");
            requireNonNegative(medical.getReimbursedAmount(), "input.medical.reimbursedAmount");
        }

        LifeInsuranceDeductionInput lifeInsurance = input.getLifeInsurance();
        if (lifeInsurance != null) {
            requireNonNegative(lifeInsurance.getNewGeneralPaidAmount(), "input.lifeInsurance.newGeneralPaidAmount");
            requireNonNegative(
                lifeInsurance.getNewIndividualPensionPaidAmount(),
                "input.lifeInsurance.newIndividualPensionPaidAmount"
            );
            requireNonNegative(lifeInsurance.getNewCareMedicalPaidAmount(), "input.lifeInsurance.newCareMedicalPaidAmount");
            requireNonNegative(lifeInsurance.getOldGeneralPaidAmount(), "input.lifeInsurance.oldGeneralPaidAmount");
            requireNonNegative(
                lifeInsurance.getOldIndividualPensionPaidAmount(),
                "input.lifeInsurance.oldIndividualPensionPaidAmount"
            );
        }

        DonationDeductionInput donation = input.getDonation();
        if (donation != null) {
            requireNonNegative(donation.getQualifiedDonationAmount(), "input.donation.qualifiedDonationAmount");
        }
    }
}
