package io.github.kmoyashi.jlaw;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.io.InputStream;

final class FixtureLoader {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private FixtureLoader() {
    }

    static JsonNode load(String name) {
        try (InputStream inputStream = FixtureLoader.class.getClassLoader().getResourceAsStream(name)) {
            if (inputStream == null) {
                throw new IllegalStateException("Fixture not found: " + name);
            }
            return MAPPER.readTree(inputStream);
        } catch (IOException e) {
            throw new IllegalStateException("Failed to load fixture: " + name, e);
        }
    }

    static StampTaxDocumentCode stampTaxDocumentCode(String value) {
        switch (value) {
            case "article1_real_estate_transfer":
                return StampTaxDocumentCode.ARTICLE1_REAL_ESTATE_TRANSFER;
            case "article1_other_transfer":
                return StampTaxDocumentCode.ARTICLE1_OTHER_TRANSFER;
            case "article1_land_lease_or_surface_right":
                return StampTaxDocumentCode.ARTICLE1_LAND_LEASE_OR_SURFACE_RIGHT;
            case "article1_consumption_loan":
                return StampTaxDocumentCode.ARTICLE1_CONSUMPTION_LOAN;
            case "article1_transportation":
                return StampTaxDocumentCode.ARTICLE1_TRANSPORTATION;
            case "article2_construction_work":
                return StampTaxDocumentCode.ARTICLE2_CONSTRUCTION_WORK;
            case "article2_general_contract":
                return StampTaxDocumentCode.ARTICLE2_GENERAL_CONTRACT;
            case "article3_bill_amount_table":
                return StampTaxDocumentCode.ARTICLE3_BILL_AMOUNT_TABLE;
            case "article3_bill_special_flat_200":
                return StampTaxDocumentCode.ARTICLE3_BILL_SPECIAL_FLAT_200;
            case "article4_security_certificate":
                return StampTaxDocumentCode.ARTICLE4_SECURITY_CERTIFICATE;
            case "article5_merger_or_split":
                return StampTaxDocumentCode.ARTICLE5_MERGER_OR_SPLIT;
            case "article6_articles_of_incorporation":
                return StampTaxDocumentCode.ARTICLE6_ARTICLES_OF_INCORPORATION;
            case "article7_continuing_transaction_basic":
                return StampTaxDocumentCode.ARTICLE7_CONTINUING_TRANSACTION_BASIC;
            case "article8_deposit_certificate":
                return StampTaxDocumentCode.ARTICLE8_DEPOSIT_CERTIFICATE;
            case "article9_transport_certificate":
                return StampTaxDocumentCode.ARTICLE9_TRANSPORT_CERTIFICATE;
            case "article10_insurance_certificate":
                return StampTaxDocumentCode.ARTICLE10_INSURANCE_CERTIFICATE;
            case "article11_letter_of_credit":
                return StampTaxDocumentCode.ARTICLE11_LETTER_OF_CREDIT;
            case "article12_trust_contract":
                return StampTaxDocumentCode.ARTICLE12_TRUST_CONTRACT;
            case "article13_debt_guarantee":
                return StampTaxDocumentCode.ARTICLE13_DEBT_GUARANTEE;
            case "article14_deposit_contract":
                return StampTaxDocumentCode.ARTICLE14_DEPOSIT_CONTRACT;
            case "article15_assignment_or_assumption":
                return StampTaxDocumentCode.ARTICLE15_ASSIGNMENT_OR_ASSUMPTION;
            case "article16_dividend_receipt":
                return StampTaxDocumentCode.ARTICLE16_DIVIDEND_RECEIPT;
            case "article17_sales_receipt":
                return StampTaxDocumentCode.ARTICLE17_SALES_RECEIPT;
            case "article17_other_receipt":
                return StampTaxDocumentCode.ARTICLE17_OTHER_RECEIPT;
            case "article18_passbook":
                return StampTaxDocumentCode.ARTICLE18_PASSBOOK;
            case "article19_misc_passbook":
                return StampTaxDocumentCode.ARTICLE19_MISC_PASSBOOK;
            case "article20_seal_book":
                return StampTaxDocumentCode.ARTICLE20_SEAL_BOOK;
            default:
                throw new IllegalArgumentException("Unsupported stamp tax document code: " + value);
        }
    }

    static StampTaxFlag stampTaxFlag(String value) {
        switch (value) {
            case "article3_copy_or_transcript_exempt":
                return StampTaxFlag.ARTICLE3_COPY_OR_TRANSCRIPT_EXEMPT;
            case "article4_specified_issuer_exempt":
                return StampTaxFlag.ARTICLE4_SPECIFIED_ISSUER_EXEMPT;
            case "article4_restricted_beneficiary_certificate_exempt":
                return StampTaxFlag.ARTICLE4_RESTRICTED_BENEFICIARY_CERTIFICATE_EXEMPT;
            case "article6_notary_copy_exempt":
                return StampTaxFlag.ARTICLE6_NOTARY_COPY_EXEMPT;
            case "article8_small_deposit_exempt":
                return StampTaxFlag.ARTICLE8_SMALL_DEPOSIT_EXEMPT;
            case "article13_identity_guarantee_exempt":
                return StampTaxFlag.ARTICLE13_IDENTITY_GUARANTEE_EXEMPT;
            case "article17_non_business_exempt":
                return StampTaxFlag.ARTICLE17_NON_BUSINESS_EXEMPT;
            case "article17_appended_receipt_exempt":
                return StampTaxFlag.ARTICLE17_APPENDED_RECEIPT_EXEMPT;
            case "article18_specified_financial_institution_exempt":
                return StampTaxFlag.ARTICLE18_SPECIFIED_FINANCIAL_INSTITUTION_EXEMPT;
            case "article18_income_tax_exempt_passbook":
                return StampTaxFlag.ARTICLE18_INCOME_TAX_EXEMPT_PASSBOOK;
            case "article18_tax_reserve_deposit_passbook":
                return StampTaxFlag.ARTICLE18_TAX_RESERVE_DEPOSIT_PASSBOOK;
            default:
                throw new IllegalArgumentException("Unsupported stamp tax flag: " + value);
        }
    }

    static WithholdingTaxCategory withholdingTaxCategory(String value) {
        switch (value) {
            case "manuscript_and_lecture":
                return WithholdingTaxCategory.MANUSCRIPT_AND_LECTURE;
            case "professional_fee":
                return WithholdingTaxCategory.PROFESSIONAL_FEE;
            case "exclusive_contract_fee":
                return WithholdingTaxCategory.EXCLUSIVE_CONTRACT_FEE;
            default:
                throw new IllegalArgumentException("Unsupported withholding tax category: " + value);
        }
    }

    static IncomeDeductionInput incomeDeductionInput(JsonNode raw) {
        IncomeDeductionInput.Builder builder = new IncomeDeductionInput.Builder(
            raw.get("total_income_amount").longValue(),
            java.time.LocalDate.parse(raw.get("date").textValue())
        );

        JsonNode spouse = raw.get("spouse");
        if (spouse != null && !spouse.isNull()) {
            builder.spouse(
                new SpouseDeductionInput(
                    spouse.get("spouse_total_income_amount").longValue(),
                    spouse.get("is_same_household").booleanValue(),
                    spouse.get("is_elderly").booleanValue()
                )
            );
        }

        JsonNode dependent = raw.get("dependent");
        if (dependent != null && !dependent.isNull()) {
            builder.dependent(
                new DependentDeductionInput(
                    dependent.path("general_count").longValue(),
                    dependent.path("specific_count").longValue(),
                    dependent.path("elderly_cohabiting_count").longValue(),
                    dependent.path("elderly_other_count").longValue()
                )
            );
        }

        builder.socialInsurancePremiumPaid(raw.path("social_insurance_premium_paid").longValue());

        JsonNode medical = raw.get("medical");
        if (medical != null && !medical.isNull()) {
            builder.medical(
                new MedicalDeductionInput(
                    medical.get("medical_expense_paid").longValue(),
                    medical.path("reimbursed_amount").longValue()
                )
            );
        }

        JsonNode lifeInsurance = raw.get("life_insurance");
        if (lifeInsurance != null && !lifeInsurance.isNull()) {
            builder.lifeInsurance(
                new LifeInsuranceDeductionInput(
                    lifeInsurance.path("new_general_paid_amount").longValue(),
                    lifeInsurance.path("new_individual_pension_paid_amount").longValue(),
                    lifeInsurance.path("new_care_medical_paid_amount").longValue(),
                    lifeInsurance.path("old_general_paid_amount").longValue(),
                    lifeInsurance.path("old_individual_pension_paid_amount").longValue()
                )
            );
        }

        JsonNode donation = raw.get("donation");
        if (donation != null && !donation.isNull()) {
            builder.donation(new DonationDeductionInput(donation.get("qualified_donation_amount").longValue()));
        }

        return builder.build();
    }
}
