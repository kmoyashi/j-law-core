package io.github.kmoyashi.jlaw;

/**
 * Categories supported by the withholding tax domain.
 */
public enum WithholdingTaxCategory {
    MANUSCRIPT_AND_LECTURE(1),
    PROFESSIONAL_FEE(2),
    EXCLUSIVE_CONTRACT_FEE(3);

    private final int code;

    WithholdingTaxCategory(int code) {
        this.code = code;
    }

    public int getCode() {
        return code;
    }

    public static WithholdingTaxCategory fromCode(int code) {
        for (WithholdingTaxCategory value : values()) {
            if (value.code == code) {
                return value;
            }
        }
        throw new IllegalArgumentException("Unsupported withholding tax category code: " + code);
    }
}
