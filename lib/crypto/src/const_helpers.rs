/// Allows writing `for` cycle in constant context.
#[macro_export]
macro_rules! const_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows writing `for` cycle in constant context, with 2 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! unroll2_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

/// Allows writing `for` cycle in constant context, with 4 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! unroll4_for {
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

/// Allows writing `for` cycle in constant context, with 6 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! unroll6_for {
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

/// Allows writing `for` cycle in constant context, with 8 stages loop unroll
/// optimization.
#[macro_export]
macro_rules! unroll8_for {
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
