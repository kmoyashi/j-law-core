use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use crate::domains::stamp_tax::policy::StampTaxPolicy;
use crate::error::InputError;
use crate::types::date::LegalDate;

/// 印紙税の文書コード。
///
/// 別表第一の税額表において税額行が変わる単位で定義する。
///
/// # 法的根拠
/// 印紙税法 別表第一
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StampTaxDocumentCode {
    Article1RealEstateTransfer,
    Article1OtherTransfer,
    Article1LandLeaseOrSurfaceRight,
    Article1ConsumptionLoan,
    Article1Transportation,
    Article2ConstructionWork,
    Article2GeneralContract,
    Article3BillAmountTable,
    Article3BillSpecialFlat200,
    Article4SecurityCertificate,
    Article5MergerOrSplit,
    Article6ArticlesOfIncorporation,
    Article7ContinuingTransactionBasic,
    Article8DepositCertificate,
    Article9TransportCertificate,
    Article10InsuranceCertificate,
    Article11LetterOfCredit,
    Article12TrustContract,
    Article13DebtGuarantee,
    Article14DepositContract,
    Article15AssignmentOrAssumption,
    Article16DividendReceipt,
    Article17SalesReceipt,
    Article17OtherReceipt,
    Article18Passbook,
    Article19MiscPassbook,
    Article20SealBook,
}

impl StampTaxDocumentCode {
    /// 永続化・外部API向けの安定文字列コード。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Article1RealEstateTransfer => "article1_real_estate_transfer",
            Self::Article1OtherTransfer => "article1_other_transfer",
            Self::Article1LandLeaseOrSurfaceRight => "article1_land_lease_or_surface_right",
            Self::Article1ConsumptionLoan => "article1_consumption_loan",
            Self::Article1Transportation => "article1_transportation",
            Self::Article2ConstructionWork => "article2_construction_work",
            Self::Article2GeneralContract => "article2_general_contract",
            Self::Article3BillAmountTable => "article3_bill_amount_table",
            Self::Article3BillSpecialFlat200 => "article3_bill_special_flat_200",
            Self::Article4SecurityCertificate => "article4_security_certificate",
            Self::Article5MergerOrSplit => "article5_merger_or_split",
            Self::Article6ArticlesOfIncorporation => "article6_articles_of_incorporation",
            Self::Article7ContinuingTransactionBasic => "article7_continuing_transaction_basic",
            Self::Article8DepositCertificate => "article8_deposit_certificate",
            Self::Article9TransportCertificate => "article9_transport_certificate",
            Self::Article10InsuranceCertificate => "article10_insurance_certificate",
            Self::Article11LetterOfCredit => "article11_letter_of_credit",
            Self::Article12TrustContract => "article12_trust_contract",
            Self::Article13DebtGuarantee => "article13_debt_guarantee",
            Self::Article14DepositContract => "article14_deposit_contract",
            Self::Article15AssignmentOrAssumption => "article15_assignment_or_assumption",
            Self::Article16DividendReceipt => "article16_dividend_receipt",
            Self::Article17SalesReceipt => "article17_sales_receipt",
            Self::Article17OtherReceipt => "article17_other_receipt",
            Self::Article18Passbook => "article18_passbook",
            Self::Article19MiscPassbook => "article19_misc_passbook",
            Self::Article20SealBook => "article20_seal_book",
        }
    }

    /// C ABI 向けの整数コード。
    pub fn ffi_code(self) -> u32 {
        match self {
            Self::Article1RealEstateTransfer => 1,
            Self::Article1OtherTransfer => 2,
            Self::Article1LandLeaseOrSurfaceRight => 3,
            Self::Article1ConsumptionLoan => 4,
            Self::Article1Transportation => 5,
            Self::Article2ConstructionWork => 6,
            Self::Article2GeneralContract => 7,
            Self::Article3BillAmountTable => 8,
            Self::Article3BillSpecialFlat200 => 9,
            Self::Article4SecurityCertificate => 10,
            Self::Article5MergerOrSplit => 11,
            Self::Article6ArticlesOfIncorporation => 12,
            Self::Article7ContinuingTransactionBasic => 13,
            Self::Article8DepositCertificate => 14,
            Self::Article9TransportCertificate => 15,
            Self::Article10InsuranceCertificate => 16,
            Self::Article11LetterOfCredit => 17,
            Self::Article12TrustContract => 18,
            Self::Article13DebtGuarantee => 19,
            Self::Article14DepositContract => 20,
            Self::Article15AssignmentOrAssumption => 21,
            Self::Article16DividendReceipt => 22,
            Self::Article17SalesReceipt => 23,
            Self::Article17OtherReceipt => 24,
            Self::Article18Passbook => 25,
            Self::Article19MiscPassbook => 26,
            Self::Article20SealBook => 27,
        }
    }

    /// C ABI の整数コードを Rust enum に戻す。
    pub fn from_ffi_code(code: u32) -> Result<Self, InputError> {
        match code {
            1 => Ok(Self::Article1RealEstateTransfer),
            2 => Ok(Self::Article1OtherTransfer),
            3 => Ok(Self::Article1LandLeaseOrSurfaceRight),
            4 => Ok(Self::Article1ConsumptionLoan),
            5 => Ok(Self::Article1Transportation),
            6 => Ok(Self::Article2ConstructionWork),
            7 => Ok(Self::Article2GeneralContract),
            8 => Ok(Self::Article3BillAmountTable),
            9 => Ok(Self::Article3BillSpecialFlat200),
            10 => Ok(Self::Article4SecurityCertificate),
            11 => Ok(Self::Article5MergerOrSplit),
            12 => Ok(Self::Article6ArticlesOfIncorporation),
            13 => Ok(Self::Article7ContinuingTransactionBasic),
            14 => Ok(Self::Article8DepositCertificate),
            15 => Ok(Self::Article9TransportCertificate),
            16 => Ok(Self::Article10InsuranceCertificate),
            17 => Ok(Self::Article11LetterOfCredit),
            18 => Ok(Self::Article12TrustContract),
            19 => Ok(Self::Article13DebtGuarantee),
            20 => Ok(Self::Article14DepositContract),
            21 => Ok(Self::Article15AssignmentOrAssumption),
            22 => Ok(Self::Article16DividendReceipt),
            23 => Ok(Self::Article17SalesReceipt),
            24 => Ok(Self::Article17OtherReceipt),
            25 => Ok(Self::Article18Passbook),
            26 => Ok(Self::Article19MiscPassbook),
            27 => Ok(Self::Article20SealBook),
            _ => Err(InputError::InvalidStampTaxInput {
                field: "document_code".into(),
                reason: format!("未知の印紙税文書コードです: {code}"),
            }),
        }
    }
}

impl fmt::Display for StampTaxDocumentCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<StampTaxDocumentCode> for u32 {
    fn from(value: StampTaxDocumentCode) -> Self {
        value.ffi_code()
    }
}

impl FromStr for StampTaxDocumentCode {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "article1_real_estate_transfer" => Ok(Self::Article1RealEstateTransfer),
            "article1_other_transfer" => Ok(Self::Article1OtherTransfer),
            "article1_land_lease_or_surface_right" => Ok(Self::Article1LandLeaseOrSurfaceRight),
            "article1_consumption_loan" => Ok(Self::Article1ConsumptionLoan),
            "article1_transportation" => Ok(Self::Article1Transportation),
            "article2_construction_work" => Ok(Self::Article2ConstructionWork),
            "article2_general_contract" => Ok(Self::Article2GeneralContract),
            "article3_bill_amount_table" => Ok(Self::Article3BillAmountTable),
            "article3_bill_special_flat_200" => Ok(Self::Article3BillSpecialFlat200),
            "article4_security_certificate" => Ok(Self::Article4SecurityCertificate),
            "article5_merger_or_split" => Ok(Self::Article5MergerOrSplit),
            "article6_articles_of_incorporation" => Ok(Self::Article6ArticlesOfIncorporation),
            "article7_continuing_transaction_basic" => Ok(Self::Article7ContinuingTransactionBasic),
            "article8_deposit_certificate" => Ok(Self::Article8DepositCertificate),
            "article9_transport_certificate" => Ok(Self::Article9TransportCertificate),
            "article10_insurance_certificate" => Ok(Self::Article10InsuranceCertificate),
            "article11_letter_of_credit" => Ok(Self::Article11LetterOfCredit),
            "article12_trust_contract" => Ok(Self::Article12TrustContract),
            "article13_debt_guarantee" => Ok(Self::Article13DebtGuarantee),
            "article14_deposit_contract" => Ok(Self::Article14DepositContract),
            "article15_assignment_or_assumption" => Ok(Self::Article15AssignmentOrAssumption),
            "article16_dividend_receipt" => Ok(Self::Article16DividendReceipt),
            "article17_sales_receipt" => Ok(Self::Article17SalesReceipt),
            "article17_other_receipt" => Ok(Self::Article17OtherReceipt),
            "article18_passbook" => Ok(Self::Article18Passbook),
            "article19_misc_passbook" => Ok(Self::Article19MiscPassbook),
            "article20_seal_book" => Ok(Self::Article20SealBook),
            _ => Err(InputError::InvalidStampTaxInput {
                field: "document_code".into(),
                reason: format!("未知の印紙税文書コードです: {s}"),
            }),
        }
    }
}

/// 印紙税の適用フラグ。
///
/// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
/// 呼び出し元が正しく判断した上で指定してください。
///
/// # 法的根拠
/// 印紙税法 別表第一
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StampTaxFlag {
    Article3CopyOrTranscriptExempt,
    Article4SpecifiedIssuerExempt,
    Article4RestrictedBeneficiaryCertificateExempt,
    Article6NotaryCopyExempt,
    Article8SmallDepositExempt,
    Article13IdentityGuaranteeExempt,
    Article17NonBusinessExempt,
    Article17AppendedReceiptExempt,
    Article18SpecifiedFinancialInstitutionExempt,
    Article18IncomeTaxExemptPassbook,
    Article18TaxReserveDepositPassbook,
}

impl StampTaxFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Article3CopyOrTranscriptExempt => "article3_copy_or_transcript_exempt",
            Self::Article4SpecifiedIssuerExempt => "article4_specified_issuer_exempt",
            Self::Article4RestrictedBeneficiaryCertificateExempt => {
                "article4_restricted_beneficiary_certificate_exempt"
            }
            Self::Article6NotaryCopyExempt => "article6_notary_copy_exempt",
            Self::Article8SmallDepositExempt => "article8_small_deposit_exempt",
            Self::Article13IdentityGuaranteeExempt => "article13_identity_guarantee_exempt",
            Self::Article17NonBusinessExempt => "article17_non_business_exempt",
            Self::Article17AppendedReceiptExempt => "article17_appended_receipt_exempt",
            Self::Article18SpecifiedFinancialInstitutionExempt => {
                "article18_specified_financial_institution_exempt"
            }
            Self::Article18IncomeTaxExemptPassbook => "article18_income_tax_exempt_passbook",
            Self::Article18TaxReserveDepositPassbook => "article18_tax_reserve_deposit_passbook",
        }
    }

    pub fn bitmask(self) -> u64 {
        match self {
            Self::Article3CopyOrTranscriptExempt => 1_u64 << 0,
            Self::Article4SpecifiedIssuerExempt => 1_u64 << 1,
            Self::Article4RestrictedBeneficiaryCertificateExempt => 1_u64 << 2,
            Self::Article6NotaryCopyExempt => 1_u64 << 3,
            Self::Article8SmallDepositExempt => 1_u64 << 4,
            Self::Article13IdentityGuaranteeExempt => 1_u64 << 5,
            Self::Article17NonBusinessExempt => 1_u64 << 6,
            Self::Article17AppendedReceiptExempt => 1_u64 << 7,
            Self::Article18SpecifiedFinancialInstitutionExempt => 1_u64 << 8,
            Self::Article18IncomeTaxExemptPassbook => 1_u64 << 9,
            Self::Article18TaxReserveDepositPassbook => 1_u64 << 10,
        }
    }

    pub fn from_bitmask(mask: u64) -> Result<HashSet<Self>, InputError> {
        let mut flags = HashSet::new();
        let mut remaining = mask;

        for flag in Self::all() {
            if remaining & flag.bitmask() != 0 {
                flags.insert(*flag);
                remaining &= !flag.bitmask();
            }
        }

        if remaining != 0 {
            return Err(InputError::InvalidStampTaxInput {
                field: "flags".into(),
                reason: format!("未知の印紙税フラグビットです: 0x{remaining:x}"),
            });
        }

        Ok(flags)
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Article3CopyOrTranscriptExempt,
            Self::Article4SpecifiedIssuerExempt,
            Self::Article4RestrictedBeneficiaryCertificateExempt,
            Self::Article6NotaryCopyExempt,
            Self::Article8SmallDepositExempt,
            Self::Article13IdentityGuaranteeExempt,
            Self::Article17NonBusinessExempt,
            Self::Article17AppendedReceiptExempt,
            Self::Article18SpecifiedFinancialInstitutionExempt,
            Self::Article18IncomeTaxExemptPassbook,
            Self::Article18TaxReserveDepositPassbook,
        ]
    }

    pub fn allowed_document_codes(self) -> &'static [StampTaxDocumentCode] {
        match self {
            Self::Article3CopyOrTranscriptExempt => {
                &[StampTaxDocumentCode::Article3BillAmountTable]
            }
            Self::Article4SpecifiedIssuerExempt
            | Self::Article4RestrictedBeneficiaryCertificateExempt => {
                &[StampTaxDocumentCode::Article4SecurityCertificate]
            }
            Self::Article6NotaryCopyExempt => {
                &[StampTaxDocumentCode::Article6ArticlesOfIncorporation]
            }
            Self::Article8SmallDepositExempt => &[StampTaxDocumentCode::Article8DepositCertificate],
            Self::Article13IdentityGuaranteeExempt => {
                &[StampTaxDocumentCode::Article13DebtGuarantee]
            }
            Self::Article17NonBusinessExempt | Self::Article17AppendedReceiptExempt => {
                &[StampTaxDocumentCode::Article17SalesReceipt]
            }
            Self::Article18SpecifiedFinancialInstitutionExempt
            | Self::Article18IncomeTaxExemptPassbook
            | Self::Article18TaxReserveDepositPassbook => {
                &[StampTaxDocumentCode::Article18Passbook]
            }
        }
    }

    pub fn requires_stated_amount(self) -> bool {
        matches!(self, Self::Article8SmallDepositExempt)
    }
}

impl fmt::Display for StampTaxFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StampTaxFlag {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "article3_copy_or_transcript_exempt" => Ok(Self::Article3CopyOrTranscriptExempt),
            "article4_specified_issuer_exempt" => Ok(Self::Article4SpecifiedIssuerExempt),
            "article4_restricted_beneficiary_certificate_exempt" => {
                Ok(Self::Article4RestrictedBeneficiaryCertificateExempt)
            }
            "article6_notary_copy_exempt" => Ok(Self::Article6NotaryCopyExempt),
            "article8_small_deposit_exempt" => Ok(Self::Article8SmallDepositExempt),
            "article13_identity_guarantee_exempt" => Ok(Self::Article13IdentityGuaranteeExempt),
            "article17_non_business_exempt" => Ok(Self::Article17NonBusinessExempt),
            "article17_appended_receipt_exempt" => Ok(Self::Article17AppendedReceiptExempt),
            "article18_specified_financial_institution_exempt" => {
                Ok(Self::Article18SpecifiedFinancialInstitutionExempt)
            }
            "article18_income_tax_exempt_passbook" => Ok(Self::Article18IncomeTaxExemptPassbook),
            "article18_tax_reserve_deposit_passbook" => {
                Ok(Self::Article18TaxReserveDepositPassbook)
            }
            _ => Err(InputError::InvalidStampTaxInput {
                field: "flags".into(),
                reason: format!("未知の印紙税フラグです: {s}"),
            }),
        }
    }
}

/// 印紙税計算のコンテキスト。
///
/// # 法的根拠
/// 印紙税法 第2条（課税文書）/ 別表第一（課税物件表）
pub struct StampTaxContext {
    /// 印紙税の文書コード。
    pub document_code: StampTaxDocumentCode,
    /// 記載金額、券面金額、受取金額などの金額。
    pub stated_amount: Option<u64>,
    /// 契約書等の作成日。
    pub target_date: LegalDate,
    /// 主な非課税文書に対応する適用フラグ。
    pub flags: HashSet<StampTaxFlag>,
    /// 特例適用判定ポリシー。
    pub policy: Box<dyn StampTaxPolicy>,
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn document_code_roundtrip() {
        let code = StampTaxDocumentCode::from_str("article17_sales_receipt").unwrap();
        assert_eq!(code, StampTaxDocumentCode::Article17SalesReceipt);
        assert_eq!(code.as_str(), "article17_sales_receipt");
        assert_eq!(
            StampTaxDocumentCode::from_ffi_code(code.ffi_code()).unwrap(),
            code
        );
    }

    #[test]
    fn flag_roundtrip_from_bitmask() {
        let mask = StampTaxFlag::Article3CopyOrTranscriptExempt.bitmask()
            | StampTaxFlag::Article17AppendedReceiptExempt.bitmask();
        let flags = StampTaxFlag::from_bitmask(mask).unwrap();
        assert!(flags.contains(&StampTaxFlag::Article3CopyOrTranscriptExempt));
        assert!(flags.contains(&StampTaxFlag::Article17AppendedReceiptExempt));
    }
}
