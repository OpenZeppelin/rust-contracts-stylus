//! Standard math utilities missing in `alloy_primitives`.
use alloy_primitives::{uint, U256, U512};

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
    /// following the selected `rounding` direction.
    ///
    /// # Arguments
    ///
    /// * `self` - first value to compute the result.
    /// * `y` - second value to compute the result.
    /// * `denominator` - denominator of the division.
    /// * `rounding` - rounding technique to use in calculation.
    #[must_use]
    fn mul_div(self, y: Self, denominator: Self, rounding: Rounding) -> Self;
}

/// Enum representing many rounding techniques.
pub enum Rounding {
    /// Rounding toward negative infinity.
    Floor,
    /// Rounding toward positive infinity.
    Ceil,
}

impl Math for U256 {
    fn sqrt(self) -> Self {
        let a = self;
        let one = U256::ONE;
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

    fn mul_div(self, y: Self, denominator: Self, rounding: Rounding) -> Self {
        assert!(
            !denominator.is_zero(),
            "division by U256::ZERO in `Math::mul_div`"
        );

        let prod = U512::from(self)
            .checked_mul(U512::from(y))
            .expect("should not panic with `U256` * `U256`");

        // Adjust for rounding if needed.
        let adjusted = match rounding {
            Rounding::Floor => prod, // No adjustment for Rounding::Floor
            Rounding::Ceil => prod
                .checked_add(U512::from(denominator) - U512::ONE)
                .expect("should not exceed `U512`"),
        };

        let result = adjusted
            .checked_div(U512::from(denominator))
            .expect("should not panic with `U512` / `U512`");

        if result > U512::from(U256::MAX) {
            panic!("should fit into `U256` in `Math::mul_div`");
        } else {
            U256::from(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{
        private::proptest::{prop_assert, prop_assume, proptest},
        uint, U256, U512,
    };

    use crate::utils::math::alloy::{Math, Rounding};

    #[test]
    fn check_sqrt_edge_cases() {
        assert_eq!(U256::ZERO.sqrt(), U256::ZERO);
        assert_eq!(U256::ONE.sqrt(), U256::ONE);
    }

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

    #[test]
    fn check_mul_div_rounding_floor() {
        proptest!(|(x: U256, y: U256, denominator: U256)| {
            prop_assume!(denominator != U256::ZERO, "division by U256::ZERO in `Math::mul_div`.");
            prop_assume!(denominator > y, "result should fit into `U256` in `Math::mul_div`.");
            let value = x.mul_div(y, denominator, Rounding::Floor);
            let expected = U512::from(x).checked_mul(U512::from(y)).expect("should not panic with `U256` * `U256`");
            let expected = expected.checked_div(U512::from(denominator)).expect("should not panic with `U512` / `U512`");
            assert_eq!(U512::from(value), expected);
        });
    }

    #[test]
    fn check_mul_div_rounding_ceil() {
        proptest!(|(x: U256, y: U256, denominator: U256)| {
            prop_assume!(denominator != U256::ZERO, "division by U256::ZERO in `Math::mul_div`.");
            prop_assume!(denominator > y, "result should fit into `U256` in `Math::mul_div`.");
            let value = x.mul_div(y, denominator, Rounding::Ceil);
            let denominator = U512::from(denominator);
            let expected = U512::from(x).checked_mul(U512::from(y)).expect("should not panic with `U256` * `U256`").checked_add(denominator - U512::ONE).expect("should not exceed `U512`");
            let expected = expected.checked_div(U512::from(denominator)).expect("should not panic with `U512` / `U512`");
            assert_eq!(U512::from(value), expected);
        });
    }

    #[test]
    fn check_mul_div_panics_when_denominator_is_zero() {
        proptest!(|(x: U256, y: U256)| {
            let result = std::panic::catch_unwind(|| {
                _ = x.mul_div(y, U256::ZERO, Rounding::Floor);
            });

            prop_assert!(result.is_err());

            // Extract and check the panic message
            let err = result.unwrap_err();
            let panic_msg = err.downcast_ref::<&str>()
                .copied()
                .or_else(|| err.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("<non-string panic>");

            prop_assert!(panic_msg.contains("division by U256::ZERO in `Math::mul_div`"));
        });
    }

    #[test]
    fn check_mul_div_panics_when_result_overflows() {
        proptest!(|(x: U256, y: U256)| {
            prop_assume!(x != U256::ZERO, "Guaranteed `x` for overflow.");
            prop_assume!(y > U256::MAX / x, "Guaranteed `y` for overflow.");

            let result = std::panic::catch_unwind(|| {
                _ = x.mul_div(y, U256::ONE, Rounding::Floor);
            });

            prop_assert!(result.is_err());

            // Extract and check the panic message
            let err = result.unwrap_err();
            let panic_msg = err.downcast_ref::<&str>()
                .copied()
                .or_else(|| err.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("<non-string panic>");

            prop_assert!(panic_msg.contains("should fit into `U256` in `Math::mul_div`"));
        });
    }
}
