//! Helper macros for implementing common traits for curve types.

/// Implements additive operations by deferring to an implementation on &Self.
#[macro_export]
macro_rules! impl_additive_ops_from_ref {
    ($type:ident, $params:ident) => {
        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Add<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                let mut result = self;
                result += &other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Add<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn add(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result += &*other;
                result
            }
        }

        impl<'b, P: $params> core::ops::Add<$type<P>> for &'b $type<P> {
            type Output = $type<P>;

            #[inline]
            fn add(self, mut other: $type<P>) -> $type<P> {
                other += self;
                other
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b, P: $params> core::ops::Add<&'a $type<P>> for &'b $type<P> {
            type Output = $type<P>;

            #[inline]
            fn add(self, other: &'a $type<P>) -> $type<P> {
                let mut result = *self;
                result += &*other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b, P: $params> core::ops::Add<&'a mut $type<P>>
            for &'b $type<P>
        {
            type Output = $type<P>;

            #[inline]
            fn add(self, other: &'a mut $type<P>) -> $type<P> {
                let mut result = *self;
                result += &*other;
                result
            }
        }

        impl<'b, P: $params> core::ops::Sub<$type<P>> for &'b $type<P> {
            type Output = $type<P>;

            #[inline]
            fn sub(self, other: $type<P>) -> $type<P> {
                let mut result = *self;
                result -= &other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b, P: $params> core::ops::Sub<&'a $type<P>> for &'b $type<P> {
            type Output = $type<P>;

            #[inline]
            fn sub(self, other: &'a $type<P>) -> $type<P> {
                let mut result = *self;
                result -= &*other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b, P: $params> core::ops::Sub<&'a mut $type<P>>
            for &'b $type<P>
        {
            type Output = $type<P>;

            #[inline]
            fn sub(self, other: &'a mut $type<P>) -> $type<P> {
                let mut result = *self;
                result -= &*other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Sub<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                let mut result = self;
                result -= &other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Sub<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn sub(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result -= &*other;
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::iter::Sum<Self> for $type<P> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::iter::Sum<&'a Self> for $type<P> {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::AddAssign<Self> for $type<P> {
            fn add_assign(&mut self, other: Self) {
                *self += &other
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::SubAssign<Self> for $type<P> {
            fn sub_assign(&mut self, other: Self) {
                *self -= &other
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::AddAssign<&'a mut Self> for $type<P> {
            fn add_assign(&mut self, other: &'a mut Self) {
                *self += &*other
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::SubAssign<&'a mut Self> for $type<P> {
            fn sub_assign(&mut self, other: &'a mut Self) {
                *self -= &*other
            }
        }
    };
}
