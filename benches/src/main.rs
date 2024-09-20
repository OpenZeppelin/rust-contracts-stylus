use benches::{access_control, erc20, erc721, merkle_proofs, report::Reports};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let reports = tokio::join!(
        access_control::bench(),
        erc20::bench(),
        erc721::bench(),
        merkle_proofs::bench()
    );

    let reports = [reports.0?, reports.1?, reports.2?, reports.3?];
    let report =
        reports.into_iter().fold(Reports::default(), Reports::merge_with);

    println!();
    println!("{report}");

    Ok(())
}
