use alloy_primitives::{address, Address, B256};
use alloy_sol_types::SolValue;
use stylus_sdk::call::{self, StaticCallContext};

/// Address of the `P256VERIFY` EVM precompile as per [RIP-7212].
///
/// [RIP-7212]: https://github.com/ethereum/RIPs/blob/723155c3d86427412b5bc0f98ad1e4791ea7347f/RIPS/rip-7212.md
pub const P256_VERIFY_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000100");

pub(crate) fn p256_verify(
    context: impl StaticCallContext,
    hash: B256,
    r: B256,
    s: B256,
    x: B256,
    y: B256,
) -> bool {
    // concatenate the input into the expected 160 bytes format
    let data = (hash, r, s, x, y).abi_encode();

    let result = call::static_call(context, P256_VERIFY_ADDRESS, &data)
        .expect("P256VERIFY precompile should not fail");

    // `P256VERIFY` returns an encoded boolean `true` for a successful
    // verification and an empty vector on a failed verification
    !result.is_empty()
}
