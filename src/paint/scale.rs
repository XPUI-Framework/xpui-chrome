//! Scaling a length by a ratio, exactly, without 64-bit division.
//!
//! A control maps a value onto pixels as `length * part / whole`. The product
//! can pass `i32::MAX` on a tall panel with a large count, so it is taken in 64
//! bits, but a 64-bit *division* is a library routine on a 32-bit core, over a
//! kilobyte of code that nothing else in a small firmware needs. The quotient
//! always fits in 32 bits here, so the long division below does it in 32-bit
//! steps, and in one native division whenever the product itself fits.

/// `length * part / whole`, rounded down, for `part` clamped to `0..=whole`.
/// Zero when `whole` or `length` is not positive.
pub(crate) fn scale(length: i32, part: i32, whole: i32) -> i32 {
    if length <= 0 || whole <= 0 {
        return 0;
    }
    // At most `length`, since `part <= whole`: it fits back in an `i32`.
    mul_div(length as u32, part.clamp(0, whole) as u32, whole as u32) as i32
}

/// `a * b / d`, rounded down, for `d > 0` and a quotient that fits in 32 bits.
pub(crate) fn mul_div(a: u32, b: u32, d: u32) -> u32 {
    let product = u64::from(a) * u64::from(b);
    let (mut high, low) = ((product >> 32) as u32, product as u32);
    if high == 0 {
        return low / d;
    }
    debug_assert!(high < d, "the quotient must fit in 32 bits");
    // Restoring division, one quotient bit per step: shift the next dividend
    // bit into the remainder, subtract `d` whenever it fits. `carry` is the
    // remainder's 33rd bit, which `d` always fits under.
    let mut quotient = 0u32;
    for bit in (0..32).rev() {
        let carry = high >> 31;
        high = (high << 1) | ((low >> bit) & 1);
        quotient <<= 1;
        if carry != 0 || high >= d {
            high = high.wrapping_sub(d);
            quotient |= 1;
        }
    }
    quotient
}

#[cfg(test)]
mod tests {
    use super::{mul_div, scale};

    #[test]
    fn matches_64_bit_arithmetic() {
        let cases = [
            (0, 0, 1),
            (7, 3, 2),
            (1448, 9_000_000, 10_000_000),
            (u32::MAX, u32::MAX, u32::MAX),
            (u32::MAX, 3, 7),
            (65_535, 65_537, 65_536),
            (123_456, 987_654, 1_000_003),
        ];
        for (a, b, d) in cases {
            let expected = (u64::from(a) * u64::from(b) / u64::from(d)) as u32;
            assert_eq!(mul_div(a, b, d), expected, "{a} * {b} / {d}");
        }
    }

    #[test]
    fn scales_a_length_and_clamps_the_part() {
        assert_eq!(scale(200, 50, 100), 100);
        assert_eq!(scale(200, 150, 100), 200, "part past whole");
        assert_eq!(scale(200, -5, 100), 0, "negative part");
        assert_eq!(scale(200, 5, 0), 0, "no whole");
        assert_eq!(scale(1448, i32::MAX, i32::MAX), 1448);
    }
}
