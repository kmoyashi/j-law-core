package io.github.kmoyashi.jlaw;

/**
 * Explicit flags used for stamp tax non-taxable or special handling cases.
 */
public enum StampTaxFlag {
    ARTICLE3_COPY_OR_TRANSCRIPT_EXEMPT(1L << 0),
    ARTICLE4_SPECIFIED_ISSUER_EXEMPT(1L << 1),
    ARTICLE4_RESTRICTED_BENEFICIARY_CERTIFICATE_EXEMPT(1L << 2),
    ARTICLE6_NOTARY_COPY_EXEMPT(1L << 3),
    ARTICLE8_SMALL_DEPOSIT_EXEMPT(1L << 4),
    ARTICLE13_IDENTITY_GUARANTEE_EXEMPT(1L << 5),
    ARTICLE17_NON_BUSINESS_EXEMPT(1L << 6),
    ARTICLE17_APPENDED_RECEIPT_EXEMPT(1L << 7),
    ARTICLE18_SPECIFIED_FINANCIAL_INSTITUTION_EXEMPT(1L << 8),
    ARTICLE18_INCOME_TAX_EXEMPT_PASSBOOK(1L << 9),
    ARTICLE18_TAX_RESERVE_DEPOSIT_PASSBOOK(1L << 10);

    private final long bitMask;

    StampTaxFlag(long bitMask) {
        this.bitMask = bitMask;
    }

    public long getBitMask() {
        return bitMask;
    }
}
