use alloy::{
    hex,
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
    contract Verifier {
        function verify(bytes32[] proof, bytes32 root, bytes32 leaf) external pure returns (bool);

        function multiProofVerify(
            bytes32[] memory proof,
            bool[] memory proofFlags,
            bytes32 root,
            bytes32[] memory leaves
        ) external pure returns (bool);
    }
);

/// Shorthand for converting from an array of hex literals to an array of
/// fixed 32-bytes slices.
macro_rules! bytes_array {
    ($($s:literal),* $(,)?) => {
        [
            $(alloy::hex!($s),)*
        ]
    };
}

const ROOT: [u8; 32] =
    hex!("bb439f2cd52c20cd5a0d2a9fc43acc94e44c58b6b8907626f15d43e2b6fa4599");
const LEAF: [u8; 32] =
    hex!("ae5a6b19bb2927169dcf59f1e0fab3ef5a58264f24afc950c8921dd0018613e1");
const PROOF: [[u8; 32]; 16] = bytes_array! {
    "ae59450ef5e421fa6543f57de8ac9a2e71072669e7dfa4b939b3cb08040c3172",
    "812723c39e2874b3ee1e91b19c0ae36cec8f16968292e5cd8d63dc820e4c880e",
    "206c4b19563946e4049473bfa67040a9ae95be0afdb147074759501ad1b7cd99",
    "9e7c6c195b5e5ea9ed0bf2a03cb9807ad610ad09d8bd19f7f2ee4972829e8e98",
    "74e58acd9bfd0778bf2d85fb4bc5532078a9519e84cae147a1794d4be857a476",
    "c4a1dadd851264dc68a0143b92933885ee987ce9bd88592fb7ef283d7e4d9b38",
    "740787cebe5fdf6a4696191d58e90964022bdc07bbcb4da85fcba3a25a310cfb",
    "06ca1871c30b7a4dea60f7739a75b3b376d4277dd15827780e4474b17cb8d42f",
    "06acc609483fc476b7b6e81b185ad5380135b3166a092b489b760f05424b8bec",
    "7164e2347a3b0349f77cdcdba42de9496fb7da1f40666a7f2e862a0ced0cf687",
    "a6e400012f156c6bf518255107bc0eefb8678d8ae4bd35b820b397edd21b45f4",
    "9cd2fab756b8e5b4a4749c472d35107d520a841c4f4a5c5c7cdebf61b299f981",
    "30b7394a87d2cb2a4fb5530a0bc78bda42d55075019f5c210c43167ba8138393",
    "af02b07c5c611f8aa1609e9962668de34a571a32d16e95bf0c90bb15cb78f019",
    "dff6a4f635ef79dec68385c4246179534dbd031e7f6ab527a25c73e46b40a7ca",
    "fd47b6c292f51911e8dfdc3e4f8bd127773b17f25b7a554beaa8741e99c41208",
};

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("MerkleProofs", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Verifier::new(contract_addr, &alice_wallet);

    let proof = PROOF.map(|h| h.into()).to_vec();

    let receipts = vec![(
        Verifier::verifyCall::SIGNATURE,
        receipt!(contract.verify(proof, ROOT.into(), LEAF.into()))?,
    )];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "merkle-proofs", None, cache_opt).await
}
