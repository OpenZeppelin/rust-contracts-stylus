use alloy_primitives::{address, uint, Address, B256, U256};
use alloy_sol_types::SolValue;
use stylus_sdk::call::{self, StaticCallContext};

/// Address of the `P256VERIFY` EVM precompile as per [RIP-7212].
///
/// [RIP-7212]: https://github.com/ethereum/RIPs/blob/723155c3d86427412b5bc0f98ad1e4791ea7347f/RIPS/rip-7212.md
pub const P256_VERIFY_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000100");

/// N/2 for excluding higher order `s` values, where
/// N is subgroup order (number of points).
pub(crate) const HALF_N: U256 = uint!(
    0x7fffffff800000007fffffffffffffffde737d56d38bcf4279dce5617e3192a8_U256
);

pub(crate) fn p256_verify(
    context: impl StaticCallContext,
    hash: B256,
    r: B256,
    s: B256,
    x: B256,
    y: B256,
) -> bool {
    // enforce low-`s` values to disallow malleability where (`r`, `s`) and
    // (`r`, `n - s`) are both valid.
    if U256::from_be_bytes(s.0) > HALF_N {
        return false;
    }

    // concatenate the input into the expected 160 bytes format
    let data = (hash, r, s, x, y).abi_encode();

    let result = call::static_call(context, P256_VERIFY_ADDRESS, &data)
        .expect("P256VERIFY precompile should not fail");

    // `P256VERIFY` returns an encoded boolean `true` for a successful
    // verification and an empty vector on a failed verification
    !result.is_empty()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use alloy_primitives::{b256, Address, B256, U256};
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::utils::precompiles::Precompiles;

    // Values from https://github.com/OffchainLabs/go-ethereum/blob/1a03a778cf634c4ed82780a5e1fad5adcd01489e/core/vm/testdata/precompiles/p256Verify.json#L3
    const VALID_HASH: B256 = b256!(
        "bb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023"
    );
    const VALID_R: B256 = b256!(
        "2ba3a8be6b94d5ec80a6d9d1190a436effe50d85a1eee859b8cc6af9bd5c2e18"
    );
    const VALID_S: B256 = b256!(
        "4cd60b855d442f5b3c7b11eb6c4e0ae7525fe710fab9aa7c77a67f79e6fadd76"
    );
    const VALID_X: B256 = b256!(
        "2927b10512bae3eddcfe467828128bad2903269919f7086069c8c4df6c732838"
    );
    const VALID_Y: B256 = b256!(
        "c7787964eaac00e5921fb1498a60f4606766b3d9685001558d1a974e7341513e"
    );
    #[entrypoint]
    #[storage]
    struct P256TestContract;

    #[public]
    impl P256TestContract {
        fn secp256r1_verify(
            &self,
            hash: B256,
            r: B256,
            s: B256,
            x: B256,
            y: B256,
        ) -> bool {
            self.p256_verify(hash, r, s, x, y)
        }
    }

    #[motsu::test]
    fn test_p256_verify_rejects_higher_s(
        contract: Contract<P256TestContract>,
        alice: Address,
    ) {
        let invalid_s: U256 = HALF_N + U256::ONE;
        let invalid_s: B256 = B256::from(invalid_s.to_be_bytes());

        // should return false before calling the `P256VERIFY` precompile.
        assert!(!contract.sender(alice).secp256r1_verify(
            VALID_HASH, VALID_R, invalid_s, VALID_X, VALID_Y
        ));
    }

    #[motsu::test]
    #[ignore = "Motsu doesn't support P256VERIFY precompile yet"]
    fn test_p256_verify(contract: Contract<P256TestContract>, alice: Address) {
        assert!(contract
            .sender(alice)
            .secp256r1_verify(VALID_HASH, VALID_R, VALID_S, VALID_X, VALID_Y));
    }

    #[motsu::test]
    #[ignore = "Motsu doesn't support P256VERIFY precompile yet"]
    fn p256_verify_returns_false_on_failed_verification(
        contract: Contract<P256TestContract>,
        alice: Address,
    ) {
        // Values from https://github.com/OffchainLabs/go-ethereum/blob/1a03a778cf634c4ed82780a5e1fad5adcd01489e/core/vm/testdata/precompiles/p256Verify.json#L10
        let hash: B256 = b256!(
            "bb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023"
        );
        let invalid_r: B256 = b256!(
            "d45c5740946b2a147f59262ee6f5bc90bd01ed280528b62b3aed5fc93f06f739"
        );
        let invalid_s: B256 = b256!(
            "b329f479a2bbd0a5c384ee1493b1f5186a87139cac5df4087c134b49156847db"
        );
        let x: B256 = b256!(
            "2927b10512bae3eddcfe467828128bad2903269919f7086069c8c4df6c732838"
        );
        let y: B256 = b256!(
            "c7787964eaac00e5921fb1498a60f4606766b3d9685001558d1a974e7341513e"
        );

        assert!(!contract
            .sender(alice)
            .secp256r1_verify(hash, invalid_r, invalid_s, x, y));
    }
}
