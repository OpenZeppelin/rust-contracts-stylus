/// Allows writing `for` cycle in constant context.
#[macro_export]
macro_rules! ct_for {
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
macro_rules! ct_for_unroll2 {
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

/// Allows writing `for` cycle in constant context, with 6 stages loop unroll
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

/// Allows writing `for` cycle in constant context, with 8 stages loop unroll
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
