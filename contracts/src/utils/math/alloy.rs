//! Standard math utilities missing in `alloy_primitives`.
use alloy_primitives::{uint, U256};

/// Trait for standard math utilities missing in `alloy_primitives`.
pub trait Math {
    /// Returns the square root of a number. If the number is not a perfect
    /// square, the value is rounded towards zero.
    /// This method is based on Newton's method for computing square roots; the
    /// algorithm is restricted to only using integer operations.
    ///
    /// # Arguments
    ///
    /// * `self` - value to perform square root operation onto.
    #[must_use]
    fn sqrt(self) -> Self;

    /// Returns the average of two numbers. The result is rounded towards zero.
    ///
    /// # Arguments
    ///
    /// * `self` - first value to compute average.
    /// * `rhs` - second value to compute average.
    #[must_use]
    fn average(self, rhs: Self) -> Self;

    /// Calculates floor(`self` * `y` / `denominator`) with full precision,
    /// following the selected `rounding` direction. Throws if result
    /// overflows a `U256` or `denominator` is zero.
    ///
    /// Original credit to Remco Bloemen under MIT license (https://xn--2-umb.com/21/muldiv) with further edits by
    /// Uniswap Labs also under MIT license.
    ///
    /// # Arguments
    ///
    /// * `self` -
    /// * `y` -
    /// * `denominator` -
    /// * `rounding` -
    #[must_use]
    fn mul_div(self, y: Self, denominator: Self, rounding: Rounding) -> Self;

    #[must_use]
    fn mul_mod(self, y: Self, m: Self) -> Self;
}

/// Enum representing many rounding techniques.
pub enum Rounding {
    /// Rounding toward negative infinity.
    Floor,
    /// Rounding toward positive infinity.
    Ceil,
    /// Rounding toward zero.
    Trunc,
    /// Rounding away from zero.
    Expand,
}

impl Rounding {
    fn unsigned_rounds_up(self) -> bool {
        // Cast the enum variant index to an integer and check if it's odd
        (self as u8) % 2 == 1
    }
}

impl Math for U256 {
    fn sqrt(self) -> Self {
        let a = self;
        let one = uint!(1_U256);
        if a <= one {
            return a;
        }

        // In this function, we use Newton's method to get a root of `f(x) := x²
        // - a`. It involves building a sequence x_n that converges
        // toward sqrt(a). For each iteration x_n, we also define the
        // error between the current value as `ε_n = | x_n - sqrt(a) |`.
        //
        // For our first estimation, we consider `e` the smallest power of 2
        // which is bigger than the square root of the target. (i.e.
        // `2**(e-1) ≤ sqrt(a) < 2**e`). We know that `e ≤ 128` because
        // `(2¹²⁸)² = 2²⁵⁶` is bigger than any uint256.
        //
        // By noticing that
        // `2**(e-1) ≤ sqrt(a) < 2**e → (2**(e-1))² ≤ a < (2**e)² → 2**(2*e-2) ≤
        // a < 2**(2*e)` we can deduce that `e - 1` is `log2(a) / 2`. We
        // can thus compute `x_n = 2**(e-1)` using a method similar to
        // the msb function.
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

        // We now have x_n such that `x_n = 2**(e-1) ≤ sqrt(a) < 2**e = 2 *
        // x_n`. This implies ε_n ≤ 2**(e-1).
        //
        // We can refine our estimation by noticing that the middle of that
        // interval minimizes the error. If we move x_n to equal
        // 2**(e-1) + 2**(e-2), then we reduce the error to ε_n ≤
        // 2**(e-2). This is going to be our x_0 (and ε_0)
        xn = (uint!(3_U256) * xn) >> 1; // ε_0 := | x_0 - sqrt(a) | ≤ 2**(e-2)

        // From here, Newton's method give us:
        // x_{n+1} = (x_n + a / x_n) / 2
        //
        // One should note that:
        // x_{n+1}² - a = ((x_n + a / x_n) / 2)² - a
        //              = ((x_n² + a) / (2 * x_n))² - a
        //              = (x_n⁴ + 2 * a * x_n² + a²) / (4 * x_n²) - a
        //              = (x_n⁴ + 2 * a * x_n² + a² - 4 * a * x_n²) / (4 * x_n²)
        //              = (x_n⁴ - 2 * a * x_n² + a²) / (4 * x_n²)
        //              = (x_n² - a)² / (2 * x_n)²
        //              = ((x_n² - a) / (2 * x_n))²
        //              ≥ 0
        // Which proves that for all n ≥ 1, sqrt(a) ≤ x_n
        //
        // This gives us the proof of quadratic convergence of the sequence:
        // ε_{n+1} = | x_{n+1} - sqrt(a) |
        //         = | (x_n + a / x_n) / 2 - sqrt(a) |
        //         = | (x_n² + a - 2*x_n*sqrt(a)) / (2 * x_n) |
        //         = | (x_n - sqrt(a))² / (2 * x_n) |
        //         = | ε_n² / (2 * x_n) |
        //         = ε_n² / | (2 * x_n) |
        //
        // For the first iteration, we have a special case where x_0 is known:
        // ε_1 = ε_0² / | (2 * x_0) |
        //     ≤ (2**(e-2))² / (2 * (2**(e-1) + 2**(e-2)))
        //     ≤ 2**(2*e-4) / (3 * 2**(e-1))
        //     ≤ 2**(e-3) / 3
        //     ≤ 2**(e-3-log2(3))
        //     ≤ 2**(e-4.5)
        //
        // For the following iterations, we use the fact that, 2**(e-1) ≤
        // sqrt(a) ≤ x_n: ε_{n+1} = ε_n² / | (2 * x_n) |
        //         ≤ (2**(e-k))² / (2 * 2**(e-1))
        //         ≤ 2**(2*e-2*k) / 2**e
        //         ≤ 2**(e-2*k)
        xn = (xn + a / xn) >> 1; // ε_1 := | x_1 - sqrt(a) | ≤ 2**(e-4.5)  -- special case, see above
        xn = (xn + a / xn) >> 1; // ε_2 := | x_2 - sqrt(a) | ≤ 2**(e-9)    -- general case with k = 4.5
        xn = (xn + a / xn) >> 1; // ε_3 := | x_3 - sqrt(a) | ≤ 2**(e-18)   -- general case with k = 9
        xn = (xn + a / xn) >> 1; // ε_4 := | x_4 - sqrt(a) | ≤ 2**(e-36)   -- general case with k = 18
        xn = (xn + a / xn) >> 1; // ε_5 := | x_5 - sqrt(a) | ≤ 2**(e-72)   -- general case with k = 36
        xn = (xn + a / xn) >> 1; // ε_6 := | x_6 - sqrt(a) | ≤ 2**(e-144)  -- general case with k = 72

        // Because e ≤ 128 (as discussed during the first estimation phase), we
        // know have reached a precision ε_6 ≤ 2**(e-144) < 1. Given
        // we're operating on integers, then we can ensure that xn is
        // now either sqrt(a) or sqrt(a) + 1.
        xn - U256::from(xn > a / xn)
    }

    fn average(self, rhs: Self) -> Self {
        // `(a + b) / 2` can overflow, so instead we compute
        // `(2 * (a & b) + (a ^ b)) / 2`.
        //
        // `a ^ b` computes the sum without carries while `2 * (a & b)` singles
        // out the carries, so `2 * (a & b) + (a ^ b) == a + b`. Sum with no
        // carries + carries.
        (self & rhs) + ((self ^ rhs) >> 1)
    }

    fn mul_mod(self, y: Self, m: Self) -> Self {
        let x = self;
        // Ensure m is not zero to avoid division by zero
        if m.is_zero() {
            panic!("Modulus cannot be zero");
        }

        let mut result = U256::ZERO;

        // x % m and y % m ensure that both inputs are reduced before any
        // computations, preventing unnecessary overhead.
        let mut base = x % m; // Reduce x modulo m
        let mut multiplier = y % m; // Reduce y modulo m

        // Modular Multiplication Loop:
        // * Add Base: If the current bit of the multiplier (y) is 1, add the
        //   base to the result modulo m.
        // * Double Base: Shift the base left (equivalent to multiplying by 2)
        //   and take modulo m to prevent overflow.
        // * Shift Multiplier: Right-shift the multiplier to process the next
        //   bit.
        while !multiplier.is_zero() {
            // If the least significant bit of the multiplier is set, add base
            // to the result.
            if multiplier & U256::from(1) != U256::ZERO {
                result = (result + base) % m;
            }

            // Double the base modulo m.
            base = (base << 1) % m;

            // Shift the multiplier to the right by 1 (equivalent to integer
            // division by 2).
            multiplier >>= 1;
        }

        result
    }

    fn mul_div(self, y: Self, denominator: Self, rounding: Rounding) -> Self {
        let one = U256::from(1);
        let two = U256::from(2);
        let three = U256::from(3);

        if denominator.is_zero() {
            panic!("Division by U256::ZERO in `Math::mul_div`")
        }

        let x = self;

        // 512-bit multiply [prod1 prod0] = x * y. Compute the product mod 2²⁵⁶
        // and mod 2²⁵⁶ - 1, then use the Chinese Remainder Theorem to
        // reconstruct the 512 bit result. The result is stored in two 256
        // variables such that product = prod1 * 2²⁵⁶ + prod0.

        // Least significant 256 bits of the product.
        let mut prod0: U256 = x * y;
        // Most significant 256 bits of the product.
        let mut prod1: U256 = {
            let mm: U256 = x.wrapping_mul(y);
            mm.wrapping_sub(prod0).wrapping_sub(U256::from((mm < prod0) as u8))
        };
        // Handle non-overflow cases, 256 by 256 division.
        if prod1.is_zero() {
            // Should not panic - denominator is not `U256::ZERO`.
            return prod0 / denominator;
        }

        // Make sure the result is less than 2²⁵⁶.
        if denominator <= prod1 {
            panic!("Under overflow in `Math::mul_div`");
        }

        ///////////////////////////////////////////////
        // 512 by 256 division.
        ///////////////////////////////////////////////

        // Make division exact by subtracting the remainder from [prod1 prod0].
        // Compute remainder using mulmod.
        let remainder: U256 = x.mul_mod(y, denominator);

        // Subtract 256 bit number from 512 bit number.
        if remainder > prod0 {
            prod1 = prod1 - one;
        }
        prod0 = prod0 - remainder;

        // Factor powers of two out of denominator and compute largest power of
        // two divisor of denominator. Always >= 1. See https://cs.stackexchange.com/q/138556/92363.
        let mut twos: U256 = denominator & (!denominator + one);
        // Divide denominator by twos.
        let denominator = denominator / twos;
        // Divide [prod1 prod0] by twos.
        prod0 = prod0 / twos;
        // Flip twos such that it is 2²⁵⁶ / twos. If twos is zero, then it
        // becomes one.
        twos =
            if twos == U256::ZERO { one } else { ((!twos + one) / twos) + one };

        // Shift in bits from prod1 into prod0.
        prod0 |= prod1 * twos;

        // Invert denominator mod 2²⁵⁶. Now that denominator is an odd number,
        // it has an inverse modulo 2²⁵⁶ such that denominator * inv ≡ 1
        // mod 2²⁵⁶. Compute the inverse by starting with a seed that is correct
        // for four bits. That is, denominator * inv ≡ 1 mod 2⁴.
        let mut inverse: U256 = (three * denominator) ^ two;

        // Use the Newton-Raphson iteration to improve the precision. Thanks to
        // Hensel's lifting lemma, this also works in modular
        // arithmetic, doubling the correct bits in each step.
        inverse = inverse * two - denominator * inverse; // inverse mod 2⁸
        inverse = inverse * two - denominator * inverse; // inverse mod 2¹⁶
        inverse = inverse * two - denominator * inverse; // inverse mod 2³²
        inverse = inverse * two - denominator * inverse; // inverse mod 2⁶⁴
        inverse = inverse * two - denominator * inverse; // inverse mod 2¹²⁸
        inverse = inverse * two - denominator * inverse; // inverse mod 2²⁵⁶

        // Because the division is now exact we can divide by multiplying with
        // the modular inverse of denominator. This will give us the
        // correct result modulo 2²⁵⁶. Since the preconditions guarantee that
        // the outcome is less than 2²⁵⁶, this is the final result. We
        // don't need to compute the high bits of the result and prod1
        // is no longer required.
        let mut result = prod0 * inverse;

        if rounding.unsigned_rounds_up()
            && x.mul_mod(y, denominator) > U256::ZERO
        {
            result = result + one;
        }

        result
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{private::proptest::proptest, uint, U256, U512};

    use crate::utils::math::alloy::Math;

    #[test]
    fn check_sqrt() {
        proptest!(|(value: U256)| {
            // U256::root(..) method requires std. Can only be used in tests.
            assert_eq!(value.sqrt(), value.root(2));
        });
    }

    #[test]
    fn check_average() {
        proptest!(|(left: U256, right: U256)| {
            // compute average in straight forward way with overflow and downcast.
            let expected = (U512::from(left) + U512::from(right)) / uint!(2_U512);
            assert_eq!(left.average(right), U256::from(expected));
        });
    }
}
