//! Official `SdI` control codes.
//!
//! Codes are five-digit decimal identifiers from the *Elenco dei controlli
//! effettuati sul file fattura* (`P3 §2`). The catalog is split by the plan into
//! codes that can be checked offline and codes that are disclosed as not checked
//! (`PLAN §5.1`, `PLAN §5.2`); all of them are representable here, including
//! codes unknown to this version of the library.
//!
//! Formatting never allocates: [`SdiCode`] writes five zero-padded digits
//! straight into the caller's formatter (`00421`).

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// The maximum number of decimal digits accepted when parsing a code.
const MAX_DIGITS: usize = 5;

/// A five-digit `SdI` control code.
///
/// The value is the numeric part of the official code: `SdiCode::from_u16(421)`
/// renders as `00421`. Unknown codes are representable and survive a JSON
/// round-trip, so a report produced by a newer ruleset can still be read and
/// re-serialized by an older consumer (`PLAN §5`, issue #12).
///
/// # Lawfulness
///
/// This type does not claim that a value appears in the official catalog. Use
/// the associated constants or [`SdiCode::ALL`] when that distinction matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SdiCode(u16);

impl SdiCode {
    /// `00001` — file name is invalid (`P3 §2.1`).
    ///
    /// Only the shape is checked offline; historical uniqueness is `00002` and
    /// requires `SdI` state (`PLAN §5.1`).
    pub const INVALID_FILE_NAME: Self = Self(1);

    /// `00002` — file name was already used (`P3 §2.1`).
    ///
    /// Requires the history of names received by the `SdI`; disclosed as not
    /// checked (`PLAN §5.2`).
    pub const DUPLICATE_FILE_NAME: Self = Self(2);

    /// `00003` — file exceeds the allowed size (`P3 §2.1`).
    pub const FILE_TOO_LARGE: Self = Self(3);

    /// `00100` — the signing certificate has expired (`P3 §2.1`).
    pub const EXPIRED_SIGNING_CERTIFICATE: Self = Self(100);

    /// `00101` — the signing certificate has been revoked (`P3 §2.1`).
    ///
    /// Requires CRL/OCSP and is disclosed as not checked (`PLAN §5.2`).
    pub const REVOKED_SIGNING_CERTIFICATE: Self = Self(101);

    /// `00102` — the signature on the file is not valid (`P3 §2.1`).
    pub const INVALID_SIGNATURE: Self = Self(102);

    /// `00103` — the signature has no timestamp reference (`P3 §2.1`).
    pub const MISSING_SIGNATURE_TIMESTAMP: Self = Self(103);

    /// `00104` — the issuing CA is not in the trusted list (`P3 §2.1`).
    ///
    /// Requires a versioned `AgID` trust store and is disclosed as not checked
    /// unless the trust store is explicitly configured (`PLAN §5.2`).
    pub const UNTRUSTED_SIGNING_CA: Self = Self(104);

    /// `00105` — the signature timestamp is after file receipt (`P3 §2.1`).
    pub const TIMESTAMP_AFTER_RECEIPT: Self = Self(105);

    /// `00106` — the file or archive is empty or corrupt (`P3 §2.1`).
    pub const EMPTY_OR_CORRUPT_FILE: Self = Self(106);

    /// `00107` — the signing certificate is not valid (`P3 §2.1`).
    pub const INVALID_SIGNING_CERTIFICATE: Self = Self(107);

    /// `00200` — the file does not conform to the schema (`P3 §2.1`).
    pub const SCHEMA_VIOLATION: Self = Self(200);

    /// `00201` — too many format errors to continue (`P3 §2.1`).
    ///
    /// The `SdI` stops after 50 format errors; the counter policy is offline.
    pub const TOO_MANY_FORMAT_ERRORS: Self = Self(201);

    /// `00300` — invalid transmitter identifier (`P3 §2.3`).
    ///
    /// Only shape and checksum are offline; existence is disclosed.
    pub const INVALID_TRANSMITTER_ID: Self = Self(300);

    /// `00301` — invalid seller VAT number (`P3 §2.3`).
    pub const INVALID_SELLER_VAT: Self = Self(301);

    /// `00302` — invalid seller tax code (`P3 §2.3`).
    pub const INVALID_SELLER_TAX_CODE: Self = Self(302);

    /// `00303` — invalid tax representative identifier (`P3 §2.3`).
    pub const INVALID_REPRESENTATIVE_ID: Self = Self(303);

    /// `00304` — invalid tax representative tax code (`P3 §2.3`).
    pub const INVALID_REPRESENTATIVE_TAX_CODE: Self = Self(304);

    /// `00305` — invalid buyer VAT number (`P3 §2.3`).
    pub const INVALID_BUYER_VAT: Self = Self(305);

    /// `00306` — invalid buyer tax code (`P3 §2.3`).
    pub const INVALID_BUYER_TAX_CODE: Self = Self(306);

    /// `00311` — recipient code is not present in the channel registry
    /// (`P3 §2.3`).
    pub const INVALID_RECIPIENT_CODE: Self = Self(311);

    /// `00312` — recipient code is not active (`P3 §2.3`).
    pub const INACTIVE_RECIPIENT_CODE: Self = Self(312);

    /// `00313` — `XXXXXXX` is allowed only for foreign recipients
    /// (`P3 §2.3`, `P3 §3.10`).
    pub const RECIPIENT_CODE_NOT_FOREIGN: Self = Self(313);

    /// `00320` — seller VAT number and tax code are inconsistent
    /// (`P3 §2.3`).
    pub const SELLER_VAT_TAX_CODE_MISMATCH: Self = Self(320);

    /// `00321` — seller tax code does not belong to a VAT group (`P3 §2.3`).
    pub const SELLER_NOT_IN_VAT_GROUP: Self = Self(321);

    /// `00322` — seller VAT group is missing its tax code (`P3 §2.3`).
    pub const SELLER_VAT_GROUP_MISSING_TAX_CODE: Self = Self(322);

    /// `00323` — seller VAT number was cancelled more than five years ago
    /// (`P3 §2.3`).
    pub const SELLER_VAT_TERMINATED: Self = Self(323);

    /// `00324` — buyer VAT number and tax code are inconsistent
    /// (`P3 §2.3`).
    pub const BUYER_VAT_TAX_CODE_MISMATCH: Self = Self(324);

    /// `00325` — buyer tax code does not belong to a VAT group (`P3 §2.3`).
    pub const BUYER_NOT_IN_VAT_GROUP: Self = Self(325);

    /// `00326` — buyer VAT group is missing its tax code (`P3 §2.3`).
    pub const BUYER_VAT_GROUP_MISSING_TAX_CODE: Self = Self(326);

    /// `00327` — buyer VAT group tax code does not identify a participant
    /// (`P3 §2.3`, new in technical specifications 1.9.1).
    pub const BUYER_VAT_GROUP_TAX_CODE_MISMATCH: Self = Self(327);

    /// `00330` — recipient PEC is a value that is not allowed, such as an
    /// `SdI` mailbox (`P3 §2.3`, local blacklist in `P3 §5.6`).
    pub const DISALLOWED_RECIPIENT_PEC: Self = Self(330);

    /// `00398` — IPA office code is not unique (`P3 §2.3`).
    pub const IPA_OFFICE_NOT_UNIQUE: Self = Self(398);

    /// `00399` — IPA recipient does not use `FPR12` (`P3 §2.3`).
    pub const IPA_RECIPIENT_FORMAT_MISMATCH: Self = Self(399);

    /// `00400` — `<Natura>` is missing while line `<AliquotaIVA>` is zero
    /// (`P3 §2.2`, `P3 §3.7`).
    pub const LINE_NATURA_MISSING_FOR_ZERO_RATE: Self = Self(400);

    /// `00401` — `<Natura>` is present while line `<AliquotaIVA>` is not zero
    /// (`P3 §2.2`, `P3 §3.7`).
    pub const LINE_NATURA_PRESENT_FOR_NONZERO_RATE: Self = Self(401);

    /// `00403` — document date is after the receipt date (`P3 §2.2`).
    ///
    /// Offline this is compared with the injected validation date (`PLAN §3`).
    pub const DOCUMENT_DATE_IN_FUTURE: Self = Self(403);

    /// `00404` — duplicate invoice (`P3 §2.2`, `P3 §3.12`).
    ///
    /// Requires the `SdI` archive and is disclosed as not checked; the
    /// intra-file equivalent is `00409`.
    pub const DUPLICATE_INVOICE: Self = Self(404);

    /// `00409` — duplicate invoice within the same batch (`P3 §3.12`).
    pub const DUPLICATE_INVOICE_IN_BATCH: Self = Self(409);

    /// `00411` — `<DatiRitenuta>` is missing while a line has
    /// `<Ritenuta>` = `SI` (`P3 §2.2`).
    pub const WITHHOLDING_BLOCK_MISSING: Self = Self(411);

    /// `00413` — social-security fund `<Natura>` is missing while its
    /// `<AliquotaIVA>` is zero (`P3 §2.2`, `P3 §3.7`).
    pub const CASSA_NATURA_MISSING_FOR_ZERO_RATE: Self = Self(413);

    /// `00414` — social-security fund `<Natura>` is present while its
    /// `<AliquotaIVA>` is not zero (`P3 §2.2`, `P3 §3.7`).
    pub const CASSA_NATURA_PRESENT_FOR_NONZERO_RATE: Self = Self(414);

    /// `00415` — social-security fund withholding is flagged but
    /// `<DatiRitenuta>` is missing (`P3 §2.2`).
    pub const CASSA_WITHHOLDING_MISSING: Self = Self(415);

    /// `00417` — neither the buyer VAT identifier nor the buyer tax code is
    /// present (`P3 §2.2`).
    pub const BUYER_ID_MISSING: Self = Self(417);

    /// `00418` — credit-note date precedes the linked invoice date
    /// (`P3 §2.2`).
    pub const CREDIT_NOTE_DATE_BEFORE_INVOICE: Self = Self(418);

    /// `00419` — no `<DatiRiepilogo>` block for one of the VAT rates
    /// (`P3 §3.1`).
    pub const RIEPILOGO_MISSING_FOR_RATE: Self = Self(419);

    /// `00420` — `N6` nature with split payment (`EsigibilitaIVA` = `S`)
    /// (`P3 §3.2`).
    pub const SPLIT_PAYMENT_WITH_N6: Self = Self(420);

    /// `00421` — `<Imposta>` does not match the expected VAT amount
    /// (`P3 §3.3`, tolerance 0.01).
    pub const WRONG_VAT_AMOUNT: Self = Self(421);

    /// `00422` — `<ImponibileImporto>` does not match the expected taxable
    /// amount (`P3 §3.4`, tolerance 1.00).
    pub const WRONG_TAXABLE_AMOUNT: Self = Self(422);

    /// `00423` — `<PrezzoTotale>` does not match quantity times unit price
    /// (`P3 §3.5`, tolerance 0.01).
    pub const WRONG_LINE_TOTAL: Self = Self(423);

    /// `00424` — a VAT rate is not expressed as a percentage (`P3 §2.2`).
    pub const VAT_RATE_NOT_PERCENTAGE: Self = Self(424);

    /// `00425` — `<Numero>` contains no numeric character (`P3 §2.2`).
    pub const DOCUMENT_NUMBER_WITHOUT_DIGITS: Self = Self(425);

    /// `00426` — recipient PEC and recipient code are inconsistent
    /// (`P3 §8.1`).
    ///
    /// Only "PEC set implies code `0000000`" is an error; the other branch is a
    /// configurable warning until the open question is resolved (`PLAN §11`).
    pub const PEC_CODE_MISMATCH: Self = Self(426);

    /// `00427` — recipient code length is incompatible with the transmission
    /// format (`P3 §3.6`).
    pub const RECIPIENT_CODE_LENGTH_MISMATCH: Self = Self(427);

    /// `00428` — transmission format does not match the root `VERSIONE`
    /// attribute (`P3 §3.6`).
    pub const VERSION_FORMAT_MISMATCH: Self = Self(428);

    /// `00429` — riepilogo `<Natura>` is missing while its `<AliquotaIVA>` is
    /// zero (`P3 §2.2`, `P3 §3.7`).
    pub const RIEPILOGO_NATURA_MISSING_FOR_ZERO_RATE: Self = Self(429);

    /// `00430` — riepilogo `<Natura>` is present while its `<AliquotaIVA>` is
    /// not zero (`P3 §2.2`, `P3 §3.7`).
    pub const RIEPILOGO_NATURA_PRESENT_FOR_NONZERO_RATE: Self = Self(430);

    /// `00437` — document-level discount has a `<Tipo>` but neither
    /// `<Percentuale>` nor `<Importo>` (`P3 §3.8`).
    pub const DOCUMENT_DISCOUNT_MISSING_VALUE: Self = Self(437);

    /// `00438` — line-level discount has a `<Tipo>` but neither
    /// `<Percentuale>` nor `<Importo>` (`P3 §3.8`).
    pub const LINE_DISCOUNT_MISSING_VALUE: Self = Self(438);

    /// `00443` — a line or fund VAT rate has no matching riepilogo rate
    /// (`P3 §3.9`).
    pub const RIEPILOGO_RATE_MISMATCH: Self = Self(443);

    /// `00444` — a line or fund nature has no matching riepilogo nature
    /// (`P3 §3.9`).
    pub const RIEPILOGO_NATURA_MISMATCH: Self = Self(444);

    /// `00445` — generic `N2`, `N3` or `N6` nature after 2021-01-01
    /// (`P3 §3.10`, `PLAN §3`).
    pub const GENERIC_NATURA_NOT_ALLOWED: Self = Self(445);

    /// `00459` — abolished regime `RF03` (`P3 §2.4`).
    ///
    /// Domain warning from the reference implementation, not an official
    /// `SdI` rejection code.
    pub const ABOLISHED_REGIME: Self = Self(459);

    /// `00460` — simplified invoice above the art. 21-bis ceiling
    /// (`P3 §2.2`, threshold needs verification per `PLAN §11`).
    pub const SIMPLIFIED_INVOICE_OVER_LIMIT: Self = Self(460);

    /// `00471` — seller and buyer must differ for this document type
    /// (`P3 §3.11`).
    pub const SELLER_EQUALS_BUYER_NOT_ALLOWED: Self = Self(471);

    /// `00472` — seller and buyer must coincide for this document type
    /// (`P3 §3.11`).
    pub const SELLER_EQUALS_BUYER_REQUIRED: Self = Self(472);

    /// `00473` — seller country is not allowed for this document type
    /// (`P3 §3.11`).
    pub const SELLER_COUNTRY_NOT_ALLOWED: Self = Self(473);

    /// `00474` — zero-rate detail lines are not allowed for this document
    /// type (`P3 §3.11`).
    pub const ZERO_RATE_LINES_NOT_ALLOWED: Self = Self(474);

    /// `00475` — buyer VAT identifier is mandatory for this document type
    /// (`P3 §3.11`).
    pub const BUYER_VAT_REQUIRED: Self = Self(475);

    /// `00476` — seller and buyer cannot both be foreign (`P3 §3.11`).
    pub const BOTH_PARTIES_FOREIGN: Self = Self(476);

    /// `00477` — invalid declaration of intent (`P3 §2.2`).
    ///
    /// Requires the `AdE` database and is disclosed as not checked
    /// (`PLAN §5.2`).
    pub const INVALID_INTENT_DECLARATION: Self = Self(477);

    /// Every code defined by this version of the library, in numeric order.
    ///
    /// The slice includes codes that are only disclosed as not checked; the
    /// distinction is expressed by [`NotChecked`](crate::diag::NotChecked) and
    /// by the rule that emits a finding.
    pub const ALL: &[Self] = &[
        Self::INVALID_FILE_NAME,
        Self::DUPLICATE_FILE_NAME,
        Self::FILE_TOO_LARGE,
        Self::EXPIRED_SIGNING_CERTIFICATE,
        Self::REVOKED_SIGNING_CERTIFICATE,
        Self::INVALID_SIGNATURE,
        Self::MISSING_SIGNATURE_TIMESTAMP,
        Self::UNTRUSTED_SIGNING_CA,
        Self::TIMESTAMP_AFTER_RECEIPT,
        Self::EMPTY_OR_CORRUPT_FILE,
        Self::INVALID_SIGNING_CERTIFICATE,
        Self::SCHEMA_VIOLATION,
        Self::TOO_MANY_FORMAT_ERRORS,
        Self::INVALID_TRANSMITTER_ID,
        Self::INVALID_SELLER_VAT,
        Self::INVALID_SELLER_TAX_CODE,
        Self::INVALID_REPRESENTATIVE_ID,
        Self::INVALID_REPRESENTATIVE_TAX_CODE,
        Self::INVALID_BUYER_VAT,
        Self::INVALID_BUYER_TAX_CODE,
        Self::INVALID_RECIPIENT_CODE,
        Self::INACTIVE_RECIPIENT_CODE,
        Self::RECIPIENT_CODE_NOT_FOREIGN,
        Self::SELLER_VAT_TAX_CODE_MISMATCH,
        Self::SELLER_NOT_IN_VAT_GROUP,
        Self::SELLER_VAT_GROUP_MISSING_TAX_CODE,
        Self::SELLER_VAT_TERMINATED,
        Self::BUYER_VAT_TAX_CODE_MISMATCH,
        Self::BUYER_NOT_IN_VAT_GROUP,
        Self::BUYER_VAT_GROUP_MISSING_TAX_CODE,
        Self::BUYER_VAT_GROUP_TAX_CODE_MISMATCH,
        Self::DISALLOWED_RECIPIENT_PEC,
        Self::IPA_OFFICE_NOT_UNIQUE,
        Self::IPA_RECIPIENT_FORMAT_MISMATCH,
        Self::LINE_NATURA_MISSING_FOR_ZERO_RATE,
        Self::LINE_NATURA_PRESENT_FOR_NONZERO_RATE,
        Self::DOCUMENT_DATE_IN_FUTURE,
        Self::DUPLICATE_INVOICE,
        Self::DUPLICATE_INVOICE_IN_BATCH,
        Self::WITHHOLDING_BLOCK_MISSING,
        Self::CASSA_NATURA_MISSING_FOR_ZERO_RATE,
        Self::CASSA_NATURA_PRESENT_FOR_NONZERO_RATE,
        Self::CASSA_WITHHOLDING_MISSING,
        Self::BUYER_ID_MISSING,
        Self::CREDIT_NOTE_DATE_BEFORE_INVOICE,
        Self::RIEPILOGO_MISSING_FOR_RATE,
        Self::SPLIT_PAYMENT_WITH_N6,
        Self::WRONG_VAT_AMOUNT,
        Self::WRONG_TAXABLE_AMOUNT,
        Self::WRONG_LINE_TOTAL,
        Self::VAT_RATE_NOT_PERCENTAGE,
        Self::DOCUMENT_NUMBER_WITHOUT_DIGITS,
        Self::PEC_CODE_MISMATCH,
        Self::RECIPIENT_CODE_LENGTH_MISMATCH,
        Self::VERSION_FORMAT_MISMATCH,
        Self::RIEPILOGO_NATURA_MISSING_FOR_ZERO_RATE,
        Self::RIEPILOGO_NATURA_PRESENT_FOR_NONZERO_RATE,
        Self::DOCUMENT_DISCOUNT_MISSING_VALUE,
        Self::LINE_DISCOUNT_MISSING_VALUE,
        Self::RIEPILOGO_RATE_MISMATCH,
        Self::RIEPILOGO_NATURA_MISMATCH,
        Self::GENERIC_NATURA_NOT_ALLOWED,
        Self::ABOLISHED_REGIME,
        Self::SIMPLIFIED_INVOICE_OVER_LIMIT,
        Self::SELLER_EQUALS_BUYER_NOT_ALLOWED,
        Self::SELLER_EQUALS_BUYER_REQUIRED,
        Self::SELLER_COUNTRY_NOT_ALLOWED,
        Self::ZERO_RATE_LINES_NOT_ALLOWED,
        Self::BUYER_VAT_REQUIRED,
        Self::BOTH_PARTIES_FOREIGN,
        Self::INVALID_INTENT_DECLARATION,
    ];

    /// Builds a code from its numeric value.
    ///
    /// Any `u16` is accepted so unknown and future codes remain representable.
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        Self(value)
    }

    /// Returns the numeric value of the code.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl From<u16> for SdiCode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl fmt::Display for SdiCode {
    /// Writes the code as five zero-padded decimal digits, for example `00421`.
    ///
    /// The formatter is written directly; no intermediate `String` is allocated.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(formatter, "{value:05}")
    }
}

impl FromStr for SdiCode {
    type Err = ParseSdiCodeError;

    /// Parses one to five ASCII decimal digits, with or without leading zeros.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return Err(ParseSdiCodeError::Empty);
        }
        if bytes.len() > MAX_DIGITS {
            return Err(ParseSdiCodeError::TooLong);
        }

        let mut code = 0_u16;
        for &byte in bytes {
            let digit = byte.wrapping_sub(b'0');
            if digit > 9 {
                return Err(ParseSdiCodeError::NonDigit);
            }
            code = code
                .checked_mul(10)
                .and_then(|current| current.checked_add(u16::from(digit)))
                .ok_or(ParseSdiCodeError::OutOfRange)?;
        }
        Ok(Self(code))
    }
}

impl Serialize for SdiCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for SdiCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SdiCodeVisitor;

        impl de::Visitor<'_> for SdiCodeVisitor {
            type Value = SdiCode;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a zero-padded SdI code string such as \"00421\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(SdiCodeVisitor)
    }
}

/// Error returned when a string is not a valid [`SdiCode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseSdiCodeError {
    /// The input was empty.
    Empty,
    /// The input had more than five digits.
    TooLong,
    /// The input contained a character that is not an ASCII digit.
    NonDigit,
    /// The numeric value does not fit in the `u16` representation.
    OutOfRange,
}

impl fmt::Display for ParseSdiCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "an SdI code cannot be empty",
            Self::TooLong => "an SdI code has at most five digits",
            Self::NonDigit => "an SdI code contains only ASCII digits",
            Self::OutOfRange => "the SdI code does not fit in a u16",
        };
        formatter.write_str(message)
    }
}

impl core::error::Error for ParseSdiCodeError {}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]

    use std::str::FromStr;

    use serde_json::json;

    use super::{ParseSdiCodeError, SdiCode};

    const EXPECTED_VALUES: [u16; 71] = [
        1, 2, 3, 100, 101, 102, 103, 104, 105, 106, 107, 200, 201, 300, 301, 302, 303, 304, 305,
        306, 311, 312, 313, 320, 321, 322, 323, 324, 325, 326, 327, 330, 398, 399, 400, 401, 403,
        404, 409, 411, 413, 414, 415, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428,
        429, 430, 437, 438, 443, 444, 445, 459, 460, 471, 472, 473, 474, 475, 476, 477,
    ];

    #[test]
    fn sdi_code_constants_match_the_plan_catalog() {
        let values: Vec<u16> = SdiCode::ALL.iter().map(|code| code.value()).collect();
        assert_eq!(values, EXPECTED_VALUES);

        assert_eq!(SdiCode::INVALID_FILE_NAME.value(), 1);
        assert_eq!(SdiCode::WRONG_VAT_AMOUNT.value(), 421);
        assert_eq!(SdiCode::INVALID_INTENT_DECLARATION.value(), 477);
        assert_eq!(SdiCode::from_u16(421), SdiCode::WRONG_VAT_AMOUNT);
        assert_eq!(SdiCode::from(421), SdiCode::WRONG_VAT_AMOUNT);
    }

    #[test]
    fn sdi_code_display_zero_pads_and_keeps_unknown_values() {
        assert_eq!(SdiCode::INVALID_FILE_NAME.to_string(), "00001");
        assert_eq!(SdiCode::WRONG_VAT_AMOUNT.to_string(), "00421");
        assert_eq!(SdiCode::from_u16(0).to_string(), "00000");
        assert_eq!(SdiCode::from_u16(9_999).to_string(), "09999");
        assert_eq!(SdiCode::from_u16(u16::MAX).to_string(), "65535");

        for code in SdiCode::ALL {
            let rendered = code.to_string();
            assert_eq!(rendered.len(), 5, "{code:?} rendered as {rendered}");
        }
    }

    #[test]
    fn sdi_code_parse_accepts_padded_short_and_unknown_forms() {
        assert_eq!(
            SdiCode::from_str("00421").unwrap(),
            SdiCode::WRONG_VAT_AMOUNT
        );
        assert_eq!(SdiCode::from_str("421").unwrap(), SdiCode::WRONG_VAT_AMOUNT);
        assert_eq!(SdiCode::from_str("00000").unwrap(), SdiCode::from_u16(0));
        assert_eq!(
            SdiCode::from_str("09999").unwrap(),
            SdiCode::from_u16(9_999)
        );
        assert_eq!(
            SdiCode::from_str("65535").unwrap(),
            SdiCode::from_u16(u16::MAX)
        );
        assert_eq!(
            "00421".parse::<SdiCode>().unwrap(),
            SdiCode::WRONG_VAT_AMOUNT
        );
    }

    #[test]
    fn sdi_code_parse_rejects_every_malformed_form() {
        assert_eq!("".parse::<SdiCode>(), Err(ParseSdiCodeError::Empty));
        assert_eq!("123456".parse::<SdiCode>(), Err(ParseSdiCodeError::TooLong));
        assert_eq!("00a21".parse::<SdiCode>(), Err(ParseSdiCodeError::NonDigit));
        assert_eq!("-0421".parse::<SdiCode>(), Err(ParseSdiCodeError::NonDigit));
        assert_eq!(
            "99999".parse::<SdiCode>(),
            Err(ParseSdiCodeError::OutOfRange)
        );
        assert_eq!(
            "65536".parse::<SdiCode>(),
            Err(ParseSdiCodeError::OutOfRange)
        );

        assert_eq!(
            ParseSdiCodeError::Empty.to_string(),
            "an SdI code cannot be empty"
        );
        assert_eq!(
            ParseSdiCodeError::TooLong.to_string(),
            "an SdI code has at most five digits"
        );
        assert_eq!(
            ParseSdiCodeError::NonDigit.to_string(),
            "an SdI code contains only ASCII digits"
        );
        assert_eq!(
            ParseSdiCodeError::OutOfRange.to_string(),
            "the SdI code does not fit in a u16"
        );
    }

    #[test]
    fn sdi_code_serde_round_trips_known_and_unknown_codes() {
        assert_eq!(
            serde_json::to_value(SdiCode::WRONG_VAT_AMOUNT).unwrap(),
            json!("00421")
        );
        assert_eq!(
            serde_json::from_value::<SdiCode>(json!("00421")).unwrap(),
            SdiCode::WRONG_VAT_AMOUNT
        );
        assert_eq!(
            serde_json::from_value::<SdiCode>(json!("09999")).unwrap(),
            SdiCode::from_u16(9_999)
        );
        assert_eq!(
            serde_json::from_value::<SdiCode>(json!("65535")).unwrap(),
            SdiCode::from_u16(u16::MAX)
        );
    }

    #[test]
    fn sdi_code_serde_rejects_invalid_values() {
        assert!(serde_json::from_value::<SdiCode>(json!("")).is_err());
        assert!(serde_json::from_value::<SdiCode>(json!("abc")).is_err());
        assert!(serde_json::from_value::<SdiCode>(json!("123456")).is_err());
        assert!(serde_json::from_value::<SdiCode>(json!("99999")).is_err());

        let type_error = serde_json::from_value::<SdiCode>(json!(421))
            .unwrap_err()
            .to_string();
        assert!(
            type_error.contains("zero-padded SdI code"),
            "unexpected error: {type_error}"
        );
    }
}
