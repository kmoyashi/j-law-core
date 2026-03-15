package io.github.kmoyashi.jlaw;

import java.time.LocalDate;

/**
 * Input used for income deduction and full income tax assessment calculations.
 */
public final class IncomeDeductionInput {
    private final long totalIncomeAmount;
    private final LocalDate date;
    private final SpouseDeductionInput spouse;
    private final DependentDeductionInput dependent;
    private final long socialInsurancePremiumPaid;
    private final MedicalDeductionInput medical;
    private final LifeInsuranceDeductionInput lifeInsurance;
    private final DonationDeductionInput donation;

    private IncomeDeductionInput(Builder builder) {
        this.totalIncomeAmount = builder.totalIncomeAmount;
        this.date = builder.date;
        this.spouse = builder.spouse;
        this.dependent = builder.dependent;
        this.socialInsurancePremiumPaid = builder.socialInsurancePremiumPaid;
        this.medical = builder.medical;
        this.lifeInsurance = builder.lifeInsurance;
        this.donation = builder.donation;
    }

    public long getTotalIncomeAmount() {
        return totalIncomeAmount;
    }

    public LocalDate getDate() {
        return date;
    }

    public SpouseDeductionInput getSpouse() {
        return spouse;
    }

    public DependentDeductionInput getDependent() {
        return dependent;
    }

    public long getSocialInsurancePremiumPaid() {
        return socialInsurancePremiumPaid;
    }

    public MedicalDeductionInput getMedical() {
        return medical;
    }

    public LifeInsuranceDeductionInput getLifeInsurance() {
        return lifeInsurance;
    }

    public DonationDeductionInput getDonation() {
        return donation;
    }

    public static final class Builder {
        private final long totalIncomeAmount;
        private final LocalDate date;
        private SpouseDeductionInput spouse;
        private DependentDeductionInput dependent = new DependentDeductionInput(0L, 0L, 0L, 0L);
        private long socialInsurancePremiumPaid;
        private MedicalDeductionInput medical;
        private LifeInsuranceDeductionInput lifeInsurance;
        private DonationDeductionInput donation;

        public Builder(long totalIncomeAmount, LocalDate date) {
            this.totalIncomeAmount = totalIncomeAmount;
            this.date = date;
        }

        public Builder spouse(SpouseDeductionInput spouse) {
            this.spouse = spouse;
            return this;
        }

        public Builder dependent(DependentDeductionInput dependent) {
            this.dependent = dependent;
            return this;
        }

        public Builder socialInsurancePremiumPaid(long socialInsurancePremiumPaid) {
            this.socialInsurancePremiumPaid = socialInsurancePremiumPaid;
            return this;
        }

        public Builder medical(MedicalDeductionInput medical) {
            this.medical = medical;
            return this;
        }

        public Builder lifeInsurance(LifeInsuranceDeductionInput lifeInsurance) {
            this.lifeInsurance = lifeInsurance;
            return this;
        }

        public Builder donation(DonationDeductionInput donation) {
            this.donation = donation;
            return this;
        }

        public IncomeDeductionInput build() {
            return new IncomeDeductionInput(this);
        }
    }
}
