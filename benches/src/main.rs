use benches::{
    access_control, eddsa, erc1155, erc1155_metadata_uri, erc20, erc6909,
    erc6909_supply, erc721, merkle_proofs, ownable, pedersen, poseidon,
    poseidon_asm_sol, poseidon_sol, report::BenchmarkReport,
};
use futures::FutureExt;
use itertools::Itertools;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let benchmarks = [
        access_control::bench().boxed(),
        erc20::bench().boxed(),
        erc721::bench().boxed(),
        merkle_proofs::bench().boxed(),
        ownable::bench().boxed(),
        erc1155::bench().boxed(),
        erc1155_metadata_uri::bench().boxed(),
        erc6909::bench().boxed(),
        erc6909_supply::bench().boxed(),
        pedersen::bench().boxed(),
        poseidon_sol::bench().boxed(),
        poseidon_asm_sol::bench().boxed(),
        poseidon::bench().boxed(),
        eddsa::bench().boxed(),
    ];

    // Run benchmarks max 3 at the same time.
    // Otherwise, nitro test node can overload and revert transaction.
    const MAX_PARALLEL: usize = 3;
    let mut report = BenchmarkReport::default();
    for chunk in &benchmarks.into_iter().chunks(MAX_PARALLEL) {
        report = futures::future::try_join_all(chunk)
            .await?
            .into_iter()
            .fold(report, BenchmarkReport::merge_with);
    }

    println!();
    println!("{report}");

    Ok(())
}
