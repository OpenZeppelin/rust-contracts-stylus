use benches::{
    access_control, erc1155, erc20, erc721, merkle_proofs, ownable,
    report::BenchmarkReport,
};
use futures::FutureExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let report = futures::future::try_join_all([
        access_control::bench().boxed(),
        erc20::bench().boxed(),
        erc721::bench().boxed(),
        merkle_proofs::bench().boxed(),
        ownable::bench().boxed(),
        erc1155::bench().boxed(),
    ])
    .await?
    .into_iter()
    .fold(BenchmarkReport::default(), BenchmarkReport::merge_with);

    println!();
    println!("{report}");

    Ok(())
}
