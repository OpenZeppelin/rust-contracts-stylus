/// A helper macro for emulating `for` loops in a `const` context.
/// # Usage
/// ```rust
/// # use ark_ff::const_for;
/// const fn for_in_const() {
///     let mut array = [0usize; 4];
///     const_for!((i in 0..(array.len())) { // We need to wrap the `array.len()` in parenthesis.
///         array[i] = i;
///     });
///     assert!(array[0] == 0);
///     assert!(array[1] == 1);
///     assert!(array[2] == 2);
///     assert!(array[3] == 3);
/// }
/// ```
#[macro_export]
macro_rules! const_for {
    (($i:ident in $start:tt..$end:tt)  $code:expr ) => {{
        let mut $i = $start;
        while $i < $end {
            $code
            $i += 1;
        }
    }};
}
