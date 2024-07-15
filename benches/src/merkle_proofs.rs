use alloy::{
    hex,
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
};
use e2e::{receipt, Account};

use crate::ArbOtherFields;

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

pub async fn bench() -> eyre::Result<()> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice).await;
    let contract = Verifier::new(contract_addr, &alice_wallet);

    let root = hex!(
        "bb439f2cd52c20cd5a0d2a9fc43acc94e44c58b6b8907626f15d43e2b6fa4599"
    );
    let leaf = hex!(
        "ae5a6b19bb2927169dcf59f1e0fab3ef5a58264f24afc950c8921dd0018613e1"
    );
    let proof = bytes_array! {
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
    }
    .map(|h| h.into())
    .to_vec();

    let receipts = vec![(
        "verify()",
        receipt!(contract.verify(proof, root.into(), leaf.into()))?,
    )];

    // Calculate the width of the longest function name.
    let max_name_width = receipts
        .iter()
        .max_by_key(|x| x.0.len())
        .expect("should at least bench one function")
        .0
        .len();
    let name_width = max_name_width.max("Merkle Proofs".len());

    // Calculate the total width of the table.
    let total_width = name_width + 3 + 6 + 3 + 6 + 3 + 20 + 4; // 3 for padding, 4 for outer borders

    // Print the table header.
    println!("+{}+", "-".repeat(total_width - 2));
    println!(
        "| {:<width$} | L2 Gas | L1 Gas |        Effective Gas |",
        "Merkle Proofs",
        width = name_width
    );
    println!(
        "|{}+--------+--------+----------------------|",
        "-".repeat(name_width + 2)
    );

    // Print each row.
    for (func_name, receipt) in receipts {
        let l2_gas = receipt.gas_used;
        let arb_fields: ArbOtherFields = receipt.other.deserialize_into()?;
        let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
        let effective_gas = l2_gas - l1_gas;

        println!(
            "| {:<width$} | {:>6} | {:>6} | {:>20} |",
            func_name,
            l2_gas,
            l1_gas,
            effective_gas,
            width = name_width
        );
    }

    // Print the table footer.
    println!("+{}+", "-".repeat(total_width - 2));

    Ok(())
}

async fn deploy(account: &Account) -> Address {
    crate::deploy(account, "merkle-proofs", None).await
}
