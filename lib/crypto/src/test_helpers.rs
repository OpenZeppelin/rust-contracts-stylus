use proptest::prelude::*;

/// Creates a proptest strategy for non-empty vectors of random bytes.
///
/// Maximum vector size is determined by proptest's default configuration.
pub(crate) fn non_empty_u8_vec_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(
        any::<u8>(),
        1..ProptestConfig::default().max_default_size_range,
    )
}
