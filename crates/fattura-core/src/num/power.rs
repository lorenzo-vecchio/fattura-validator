//! Power-of-ten helper for scale conversion.

/// Returns `10^exponent`.
///
/// Every caller validates `exponent <= 8` before calling, so `10^8` is the
/// largest value produced and the computation cannot overflow the `i128`
/// representation.
pub(super) fn pow10(exponent: u8) -> i128 {
    10i128.pow(u32::from(exponent))
}
