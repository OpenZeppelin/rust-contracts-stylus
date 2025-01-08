use benches::{
    access_control, erc1155, erc1155_metadata_uri, erc20, erc4626, erc721,
    merkle_proofs, ownable, poseidon, poseidon_sol, report::BenchmarkReport,
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
        erc4626::bench().boxed(),
        poseidon::bench().boxed(),
        poseidon_sol::bench().boxed(),
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
