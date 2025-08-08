//! This module contains helpers for functions with constant context, like
//! [`ct_for`] - constant time `for` cycle, as well as its optimized versions
//! like [`ct_for_unroll6`], that performs [loop unroll] optimization and can be
//! used both from compile time and runtime.
//!
//! Beware of using an optimized version everywhere, since it can bloat
//! binary (WASM) size easily.
//! Measure impact first.
//!
//! [loop unroll]: https://en.wikipedia.org/wiki/Loop_unrolling

/// Allows `for` loops in constant context.
///
/// ```rust,ignore
/// // This loop runs from 0 to 10 (not including 10).
/// ct_for!((i in 0..10) {
///     // loop body here
/// });
/// ```
#[macro_export]
macro_rules! ct_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows reversed `for` loop in constant context.
///
/// Start (`$start`) index is not inclusive.
///
/// ```rust,ignore
/// // This loop:
/// ct_rev_for!((i in 0..10) {
///     // loop body here
/// });
/// // is similar to the following loop:
/// for i in (0..10).rev() {
///     // loop body here
/// }
/// // Will runs from 9 till 0 (not including 0).
/// ```
#[macro_export]
macro_rules! ct_rev_for {
    (($i:ident in $end:tt.. $start:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::rev_cycle!($i, $end, $code);
        }
    }};
}

/// Allows `for` loop in constant context, with 2 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! ct_for_unroll2 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows `for` loop in constant context, with 4 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! ct_for_unroll4 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows `for` loop in constant context, with 6 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! ct_for_unroll6 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows reversed `for` loop in constant context, with 6 stages loop
/// unroll optimization.
///
/// Start (`$start`) index is not inclusive.
#[macro_export]
macro_rules! ct_rev_for_unroll6 {
    (($i:ident in $end:tt.. $start:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
        }
    }};
}

/// Allows `for` loop in constant context, with 8 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! ct_for_unroll8 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Single cycle step in the loop.
#[macro_export]
macro_rules! cycle {
    ($i:ident, $end:tt, $code:expr) => {{
        if $i < $end {
            $code
        } else {
            break;
        }
        $i += 1;
    }};
}

/// Single cycle step back in the loop.
#[macro_export]
macro_rules! rev_cycle {
    ($i:ident, $end:tt, $code:expr) => {{
        if $end < $i {
            $i -= 1;
            $code
        } else {
            break;
        }
    }};
}
