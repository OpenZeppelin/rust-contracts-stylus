use alloy_primitives::{uint, U256};

// TODO#q: use more smart way

/**
 * @dev Returns the square root of a number. If the number is not a perfect
 * square, the value is rounded towards zero.
 *
 * This method is based on Newton's method for computing square roots; the
 * algorithm is restricted to only using integer operations.
 */
pub fn sqrt(a: U256) -> U256 {
    // TODO#q: refactor this
    let one = uint!(1_U256);
    if a <= one {
        return a;
    }

    let mut aa = a;
    let mut xn = one;

    if aa >= (one << 128) {
        aa >>= 128;
        xn <<= 64;
    }
    if aa >= (one << 64) {
        aa >>= 64;
        xn <<= 32;
    }
    if aa >= (one << 32) {
        aa >>= 32;
        xn <<= 16;
    }
    if aa >= (one << 16) {
        aa >>= 16;
        xn <<= 8;
    }
    if aa >= (one << 8) {
        aa >>= 8;
        xn <<= 4;
    }
    if aa >= (one << 4) {
        aa >>= 4;
        xn <<= 2;
    }
    if aa >= (one << 2) {
        xn <<= 1;
    }

    xn = (uint!(3_U256) * xn) >> 1;

    xn = (xn + a / xn) >> 1;
    xn = (xn + a / xn) >> 1;
    xn = (xn + a / xn) >> 1;
    xn = (xn + a / xn) >> 1;
    xn = (xn + a / xn) >> 1;
    xn = (xn + a / xn) >> 1;

    xn - U256::from(xn > a / xn)
}

/**
 * @dev Returns the average of two numbers. The result is rounded towards
 * zero.
 */
pub fn average(a: U256, b: U256) -> U256 {
    (a & b) + (a ^ b) / uint!(2_U256)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::uint;

    use crate::utils::math::sqrt;

    #[test]
    fn check_sqrt() {
        // TODO#q: use proptest
        assert_eq!(sqrt(uint!(27_U256)), uint!(5_U256));
    }
}
